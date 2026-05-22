use std::sync::{Arc, Mutex, atomic::AtomicUsize};

use super::common::{default_progress_n, thousands_format};
use crate::transformations::prelude::*;

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
    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let inner = self
            .toml_value
            .value
            .as_ref()
            .expect("can't call declare_output_files when validation failed");
        let infix = inner
            .output_infix
            .as_ref()
            .expect("output_infix must be set in config");
        vec![OutputDeclaration {
            id: "progress".to_string(),
            target: WriteTargetConfig::new(vec![infix.clone()], "progress".to_string()),
            sink_config: SinkConfig::default(),
            format: fastqrab_io::FileFormat::Text,
            chunk_policy: ChunkPolicy::default(),
            bam_options: None,
            singleton: true,
            span: inner.output_infix.span(),
        }]
    }
}

impl Step for Progress {
    fn init(
        &mut self,
        input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
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
            "Thread config: per_input_segment {}, processing: {}, per_output_file: {}",
            input_info.threading_configuration.n_input_per_segment,
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
        _input_info: &InputInfo,
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
        let offset = counter % self.n;
        //todo: Why are we printing multiple times per block?
        for ii in ((counter + offset)..next).step_by(self.n) {
            let elapsed = self
                .start_time
                .expect("start_time must be set when processing blocks")
                .elapsed()
                .as_secs_f64();
            let rate_total = ii as f64 / elapsed;
            let msg: String = if elapsed > 1.0 {
                // cov:excl-start hard to trigger in tests without slowing everything down
                format!(
                    "Processed Total: {} ({:} molecules/s), Elapsed: {}s",
                    thousands_format(ii as f64, 0),
                    thousands_format(rate_total, 2),
                    self.start_time
                        .expect("start_time must be set when processing blocks")
                        .elapsed()
                        .as_secs()
                )
                // cov:excl-end
            } else {
                format!(
                    "Processed Total: {}, Elapsed: {}s",
                    thousands_format(ii as f64, 0),
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
        let count: usize = self.total_count.load(std::sync::atomic::Ordering::SeqCst) as usize;
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

    fn post_finalize(&self) {
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
        self.output(&msg).ok(); //swallow error. If it fails here, we ignore that

        if let Some(writer) = self.writer.lock().expect("poisoned").take() {
            let _ = writer.finish().ok(); //we choose to ignore if finishing the progress writer
            //fails (disk full?)
        }
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
