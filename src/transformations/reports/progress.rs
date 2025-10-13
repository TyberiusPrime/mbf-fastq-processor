use super::super::{FinalizeReportResult, InputInfo, Step, Transformation};
use super::common::{default_progress_n, thousands_format};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use anyhow::{bail, Result};
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub total_count: Arc<Mutex<usize>>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    #[serde(default = "default_progress_n")]
    pub n: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    pub output_infix: Option<String>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filename: Option<PathBuf>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    lock: Arc<Mutex<()>>,
}

impl Progress {
    pub fn output(&self, msg: &str) {
        let _guard = self.lock.lock().unwrap();
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
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if let Some(output) = output_def.as_ref() {
            if output.stdout && self.output_infix.is_none() {
                bail!(
                    "Can't output to stdout and log progress to stdout. Supply an output_infix to Progress"
                );
            }
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        if let Some(output_infix) = &self.output_infix {
            self.filename =
                Some(output_directory.join(format!("{output_prefix}_{output_infix}.progress")));
            if self.filename.as_ref().unwrap().exists() {
                bail!("Progress file {} already exists. Please remove it or choose a different output_infix", self.filename.as_ref().unwrap().display());
            }
            //create empty file so we are sure we can write there
            let _ = ex::fs::File::create(self.filename.as_ref().unwrap())?;
        }
        Ok(None)
    }

    #[allow(clippy::cast_precision_loss)]
    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if self.start_time.is_none() {
            self.start_time = Some(std::time::Instant::now());
        }
        let (counter, next) = {
            let mut counter = self.total_count.lock().unwrap();
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
            let elapsed = self.start_time.unwrap().elapsed().as_secs_f64();
            let rate_total = ii as f64 / elapsed;
            let msg: String = if elapsed > 1.0 {
                format!(
                    "Processed Total: {} ({:.2} molecules/s), Elapsed: {}s",
                    thousands_format(ii as f64, 0),
                    thousands_format(rate_total, 2),
                    self.start_time.unwrap().elapsed().as_secs()
                )
            } else {
                format!(
                    "Processed Total: {}, Elapsed: {}s",
                    ii,
                    self.start_time.unwrap().elapsed().as_secs()
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
    fn finalize(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let elapsed = self.start_time.unwrap().elapsed().as_secs_f64();
        let count: usize = *self.total_count.lock().unwrap();
        let msg = format!(
            "Took {:.2} s ({}) to process {} molecules for an effective rate of {:.2} molecules/s",
            elapsed,
            crate::format_seconds_to_hhmmss(elapsed as u64),
            count,
            count as f64 / elapsed
        );
        self.output(&msg);

        Ok(None)
    }
}
