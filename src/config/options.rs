#![allow(clippy::struct_field_names)] // FailureOptions - eserde(?) interferes with clippy here. 
use crate::io::output::compressed_output::{SimulatedWriteError, SimulatedWriteFailure};
use anyhow::{Context, Result};
use schemars::JsonSchema;

#[derive(eserde::Deserialize, Debug, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FailureOptions {
    #[serde(default)]
    pub fail_output_after_bytes: Option<usize>,
    #[serde(default)]
    pub fail_output_error: Option<FailOutputError>,
    #[serde(default)]
    pub fail_output_raw_os_code: Option<i32>,
}

impl FailureOptions {
    pub fn simulated_output_failure(&self) -> Result<Option<SimulatedWriteFailure>> {
        let Some(remaining_bytes) = self.fail_output_after_bytes else {
            return Ok(None);
        };

        let failure_error = self
            .fail_output_error
            .clone()
            .unwrap_or(FailOutputError::DiskFull);
        let error = match failure_error {
            FailOutputError::DiskFull => SimulatedWriteError::RawOs(28),
            FailOutputError::Other => SimulatedWriteError::Other,
            FailOutputError::RawOs => {
                let code = self
                    .fail_output_raw_os_code
                    .context(
                        "options.debug_failures.fail_output_raw_os_code required when fail_output_error = 'raw_os'",
                    )?;
                SimulatedWriteError::RawOs(code)
            }
        };

        Ok(Some(SimulatedWriteFailure {
            remaining_bytes: Some(remaining_bytes),
            error,
        }))
    }
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailOutputError {
    DiskFull,
    Other,
    RawOs,
}

fn default_thread_count() -> usize {
    //num_cpus::get()
    2
}

pub fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

fn default_output_buffer_size() -> usize {
    1024 * 1024 // bytes, per fastq input file
}

pub fn default_block_size() -> usize {
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

fn default_spot_check_read_pairing() -> bool {
    true
}

fn default_pipeline_mode() -> PipelineMode {
    PipelineMode::ThreadBased
}

#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PipelineMode {
    /// Original thread-per-stage model
    ThreadBased,
    /// Tokio async tasks
    Async,
    /// Coordinator thread with work pool
    Coordinator,
    /// Simplified single-queue coordinator
    CoordinatorSimple,
}

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_output_buffer_size")]
    pub output_buffer_size: usize,
    #[serde(default)]
    pub accept_duplicate_files: bool,
    #[serde(default = "default_spot_check_read_pairing")]
    pub spot_check_read_pairing: bool,
    #[serde(default = "default_pipeline_mode")]
    pub pipeline_mode: PipelineMode,
    #[serde(default)]
    pub debug_failures: FailureOptions,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            thread_count: 10,
            block_size: default_block_size(),
            buffer_size: default_buffer_size(),
            output_buffer_size: default_output_buffer_size(),
            accept_duplicate_files: false,
            spot_check_read_pairing: default_spot_check_read_pairing(),
            pipeline_mode: default_pipeline_mode(),
            debug_failures: FailureOptions::default(),
        }
    }
}
