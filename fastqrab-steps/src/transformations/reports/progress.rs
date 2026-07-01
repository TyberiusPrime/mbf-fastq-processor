use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use super::common::{default_progress_n, thousands_format};

/// Trailing wall-clock window (seconds) over which the "current" rate is
/// averaged, to smooth out the lumpiness of block-sized arrivals.
const RATE_WINDOW_SECONDS: f64 = 1.0;
use crate::transformations::prelude::*;
use crate::verify_path_component;

fn format_seconds_to_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{hours:02}:{minutes:02}:{secs:02}")
}

/// output a progress indicator
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Progress {
    #[schemars(skip)]
    #[tpd(skip, default)]
    pub total_count: Arc<AtomicUsize>,

    #[schemars(skip)]
    #[tpd(skip, default)]
    pub start_time: Option<std::time::Instant>,

    /// Trailing samples of (`elapsed_secs`, `count`) used to compute the "current"
    /// rate over a wall-clock window rather than between two adjacent reports.
    #[schemars(skip)]
    #[tpd(skip, default)]
    rate_samples: Arc<Mutex<VecDeque<(f64, usize)>>>,

    pub n: usize,
    pub output_infix: String,

    #[schemars(skip)]
    #[tpd(skip, default)]
    pub finalize_timepoint: Arc<Mutex<Option<std::time::Instant>>>,

    #[schemars(skip)]
    #[tpd(skip, default)]
    writer: Arc<Mutex<Option<ChunkedRecordWriter>>>,
}

impl VerifyIn<PartialConfig> for PartialProgress {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.n.or_with(default_progress_n);
        self.output_infix.or_with(|| "--stdout--".to_string());
        self.output_infix.verify(verify_path_component);
        self.finalize_timepoint = Some(Arc::new(Mutex::new(None)));
        Ok(())
    }
}

impl Progress {
    pub fn output(&self, msg: &str) -> Result<()> {
        let mut guard = self
            .writer
            .lock()
            .expect("writer lock must not be poisoned");
        let writer = guard.as_mut().expect("Writer set in init");
        let mut bytes = msg.as_bytes().to_vec();
        bytes.push(b'\n');
        writer
            .write_text_record(&bytes)
            .context("failed to write to progress file")?;
        writer.flush().context("failed to flush progress file")?;
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialProgress> {
    fn declare_output_files(&self) -> Option<Vec<OutputDeclaration>> {
        if let Some(inner) = self.toml_value.as_ref() {
            let infix = inner
                .output_infix
                .as_ref()
                .expect("output_infix must be set in config");
            Some(vec![OutputDeclaration {
                id: "progress".to_string(),
                target: WriteTargetConfig::new(vec![infix.clone()], None, "progress".to_string()),
                sink_config: SinkConfig::default(),
                format: fastqrab_io::FileFormat::Text,
                chunk_policy: ChunkPolicy::default(),
                bam_options: None,
                singleton: true,
                span: inner.output_infix.span(),
            }])
        } else {
            Some(vec![]) //there should be output files, but we can't name them.
        }
    }
}

impl Step for Progress {
    fn init(
        &mut self,
        input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
        _input_files: &mut StepInputFiles,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Get the single (tag-0) writer for this non-demultiplexed output
        let mut per_tag = output_files.take("progress");
        let writer = per_tag
            .remove(&0)
            .expect("tag 0 writer must exist for progress output");
        *self.writer.lock().expect("poisoned") = Some(writer);
        self.start_time = Some(std::time::Instant::now());
        // report thread configuration
        self.output(&format!(
            "Thread config: per_input_segment {} (+ {} pod-demux), processing: {}, per_output_file: {}",
            input_info.threading_configuration.n_input_per_segment,
            input_info.threading_configuration.n_pod_demux_per_segment,
            input_info.threading_configuration.n_processing,
            input_info.threading_configuration.n_output,
        ))?;
        Ok(None)
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "Number of reads is not going to be that high. And if it is, the loss of precision is ok."
    )]
    #[mutants::skip] // we're not testing number values
    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let (counter, next) = {
            let len = block.len();
            let val = self
                .total_count
                .fetch_add(len, std::sync::atomic::Ordering::Relaxed);
            let next = val + len;
            (val, next)
        };
        // Print at most one progress line per block: on the first block, and
        // thereafter whenever the running total crosses a multiple of `n`.
        // (The old code looped over every crossed multiple within a block,
        // emitting near-identical lines microseconds apart and making the line
        // count depend on the — now irregular — block sizes.) Each multiple of
        // `n` falls in exactly one block, so this is deterministic in the total
        // count regardless of how blocks are chunked.
        if next > counter && (counter == 0 || next / self.n != counter / self.n) {
            let ii = next;
            let elapsed = self
                .start_time
                .expect("start_time must be set when processing blocks")
                .elapsed()
                .as_secs_f64();
            let rate_total = ii as f64 / elapsed;
            // "Current" rate averaged over a trailing wall-clock window. Reports
            // fire per-molecule-count, not per-tick, and blocks arrive lumpily
            // (often several report points share near-identical timestamps), so
            // a two-point delta is extremely noisy. Averaging over the window
            // smooths that out while still tracking recent throughput.
            let rate_current = {
                let mut samples = self.rate_samples.lock().expect("poisoned");
                samples.push_back((elapsed, ii));
                // Drop samples older than the window, but always keep at least
                // two so there is an interval to measure even on slow streams.
                while samples.len() > 2 && samples[1].0 < elapsed - RATE_WINDOW_SECONDS {
                    samples.pop_front(); // cov:excl-line not happening in test
                } // cov:excl-line
                let (front_elapsed, front_ii) = samples[0];
                let dt = elapsed - front_elapsed;
                let dn = ii as f64 - front_ii as f64;
                if dt > f64::EPSILON && dn >= 0.0 {
                    dn / dt
                } else {
                    // single sample, or out-of-order reports across threads
                    rate_total
                }
            };
            let in_flight = input_info
                .blocks_in_flight
                .load(std::sync::atomic::Ordering::Relaxed);
            let msg: String = if elapsed > 1.0 {
                // cov:excl-start hard to trigger in tests without slowing everything down
                format!(
                    "Processed Total: {:>15} ({:>15} molecules/s current, {:>15} molecules/s cumulative), in-flight: {:>4}, Elapsed: {:>6}s",
                    thousands_format(ii as f64, 0),
                    thousands_format(rate_current, 2),
                    thousands_format(rate_total, 2),
                    in_flight,
                    self.start_time
                        .expect("start_time must be set when processing blocks")
                        .elapsed()
                        .as_secs()
                )
                // cov:excl-end
            } else {
                format!(
                    "Processed Total: {:>15}, in-flight: {:>4}, Elapsed: {:>6}s",
                    thousands_format(ii as f64, 0),
                    in_flight,
                    self.start_time
                        .expect("start_time must be set when processing blocks")
                        .elapsed()
                        .as_secs()
                )
            };
            self.output(&msg)?;
        }
        //not quite deterministic since it might come before or after other blocks-in-flight
        if block.is_final {
            self.output("Final block passed Progress stage.")?;
        }
        Ok((block, true))
    }

    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "loss of precision is ok for giant read counts"
    )]
    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let elapsed = self
            .start_time
            .unwrap_or_else(std::time::Instant::now)
            .elapsed()
            .as_secs_f64();
        let count: usize = self.total_count.load(std::sync::atomic::Ordering::SeqCst);
        let msg = format!(
            "Took {:.2} s ({}) to process {} molecules for an effective rate of {:} molecules/s",
            elapsed,
            format_seconds_to_hhmmss(elapsed as u64),
            thousands_format(count as f64, 0),
            thousands_format(count as f64 / elapsed, 2),
        );
        self.output(&msg)?;

        self.finalize_timepoint
            .lock()
            .expect("poisoned")
            .replace(std::time::Instant::now());

        Ok(None)
    }

    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        reason = "don't run it for more than 2^53 seconds^^"
    )]
    fn post_finalize(&self, _reports: &[FinalizeReportResult]) -> Result<()> {
        let elapsed = self
            .finalize_timepoint
            .lock()
            .expect("poisoned")
            .unwrap_or_else(std::time::Instant::now)
            .elapsed()
            .as_secs_f64();
        let msg = format!(
            "Finalizing all steps took {:.2} s ({}).",
            elapsed,
            format_seconds_to_hhmmss(elapsed as u64)
        );
        let _ = self.output(&msg); //swallow error. If it fails here, we ignore that

        if let Some(writer) = self.writer.lock().expect("poisoned").take() {
            let _ = writer.finish().ok(); //we choose to ignore if finishing the progress writer
            //fails (disk full?)
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_format_seconds_to_hhmmss() {
        assert_eq!(super::format_seconds_to_hhmmss(0), "00:00:00");
        assert_eq!(super::format_seconds_to_hhmmss(59), "00:00:59");
        assert_eq!(super::format_seconds_to_hhmmss(60), "00:01:00");
        assert_eq!(super::format_seconds_to_hhmmss(3599), "00:59:59");
        assert_eq!(super::format_seconds_to_hhmmss(3600), "01:00:00");
        assert_eq!(super::format_seconds_to_hhmmss(3601), "01:00:01");
        assert_eq!(super::format_seconds_to_hhmmss(3661), "01:01:01");
        assert_eq!(super::format_seconds_to_hhmmss(86399), "23:59:59");
        assert_eq!(super::format_seconds_to_hhmmss(86400), "24:00:00");
        assert_eq!(super::format_seconds_to_hhmmss(86400 * 10), "240:00:00");
    }
}
