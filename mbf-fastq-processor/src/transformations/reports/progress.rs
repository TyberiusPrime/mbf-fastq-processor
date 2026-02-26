use crate::transformations::prelude::*;

use super::common::{default_progress_n, thousands_format};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

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
    pub total_count: Arc<Mutex<usize>>,
    #[schemars(skip)]
    #[tpd(skip, default)]
    pub start_time: Option<std::time::Instant>,
    pub n: usize,
    pub output_infix: Option<String>,

    #[schemars(with = "String")]
    pub filename: Option<PathBuf>,

    //output lock
    #[schemars(skip)]
    #[tpd(skip, default)]
    lock: Arc<Mutex<()>>,
}

impl VerifyIn<PartialConfig> for PartialProgress {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.n.or_with(default_progress_n);
        let stdout = parent
            .output
            .as_ref()
            .and_then(|x| x.as_ref())
            .and_then(|o| o.stdout.as_ref())
            .copied()
            .unwrap_or(false);
        let has_output_infix = self
            .output_infix
            .as_ref()
            .and_then(|x| x.as_ref())
            .is_some();
        if stdout && !has_output_infix {
            self.output_infix.state = TomlValueState::ValidationFailed {
                message: "Missing output_infix".to_string(),
            };
            self.output_infix.help = Some(
                "Supply an output_infix to write progress to a file instead of stdout (which is used by [output] already).".to_string(),
            );
        }
        Ok(())
    }
}

impl Progress {
    pub fn output(&self, msg: &str) {
        let _guard = self.lock.lock().expect("lock must not be poisoned");
        if let Some(filename) = self.filename.as_ref() {
            let mut report_file = ex::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(filename)
                .expect("failed to open progress file");
            writeln!(report_file, "{msg}").expect("failed to write to progress file");
        } else {
            println!("{msg}");
        }
    }
}

impl Step for Progress {
    // it actually doesn't. Since we're using a lock interneally.
    // fn needs_serial(&self) -> bool {
    //     false
    // }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        if let Some(output_infix) = &self.output_infix {
            let base =
                crate::join_nonempty([output_prefix, output_infix.as_str()], output_ix_separator);
            self.filename = Some(output_directory.join(format!("{base}.progress")));

            crate::output::ensure_output_destination_available(
                self.filename
                    .as_ref()
                    .expect("filename must be set when output_infix is provided"),
                allow_overwrite,
            )?;

            //create empty file so we are sure we can write there
            let _ = ex::fs::File::create(
                self.filename
                    .as_ref()
                    .expect("filename must be set when output_infix is provided"),
            )?;
        }
        self.start_time = Some(std::time::Instant::now());
        Ok(None)
    }

    #[allow(clippy::cast_precision_loss)]
    #[mutants::skip] // we're not testing number values
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let (counter, next) = {
            let mut counter = self
                .total_count
                .lock()
                .expect("total_count lock must not be poisoned");
            //    println!("Thread {:?}", thread::current().id());
            let val = *counter;
            let next = *counter + block.len();
            *counter = next;
            drop(counter);
            (val, next)
        };
        //now for any multiple of n that's in the range, we print a message
        let offset = counter % self.n;
        for ii in ((counter + offset)..next).step_by(self.n) {
            let elapsed = self
                .start_time
                .expect("start_time must be set when processing blocks")
                .elapsed()
                .as_secs_f64();
            let rate_total = ii as f64 / elapsed;
            let msg: String = if elapsed > 1.0 {
                format!(
                    "Processed Total: {} ({:} molecules/s), Elapsed: {}s",
                    thousands_format(ii as f64, 0),
                    thousands_format(rate_total, 2),
                    self.start_time
                        .expect("start_time must be set when processing blocks")
                        .elapsed()
                        .as_secs()
                )
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
            self.output(&msg);
        }
        Ok((block, true))
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let elapsed = self
            .start_time
            .unwrap_or_else(std::time::Instant::now)
            .elapsed()
            .as_secs_f64();
        let count: usize = *self
            .total_count
            .lock()
            .expect("total_count lock must not be poisoned");
        let msg = format!(
            "Took {:.2} s ({}) to process {} molecules for an effective rate of {:} molecules/s",
            elapsed,
            format_seconds_to_hhmmss(elapsed as u64),
            thousands_format(count as f64, 0),
            thousands_format(count as f64 / elapsed, 2),
        );
        self.output(&msg);

        Ok(None)
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
