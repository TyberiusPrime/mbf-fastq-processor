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

#[must_use]
pub fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

fn default_output_buffer_size() -> usize {
    1024 * 1024 // bytes, per fastq input file
}

#[must_use]
pub fn default_block_size() -> usize {
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

fn default_spot_check_read_pairing() -> bool {
    true
}

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub thread_count: Option<usize>,
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
    #[serde(default)]
    pub debug_failures: FailureOptions,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            thread_count: None,
            block_size: default_block_size(),
            buffer_size: default_buffer_size(),
            output_buffer_size: default_output_buffer_size(),
            accept_duplicate_files: false,
            spot_check_read_pairing: default_spot_check_read_pairing(),
            debug_failures: FailureOptions::default(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_options_deserialize_missing_vs_empty() {
        // Test that missing [options] section and empty [options] section
        // produce the same result

        // Config with no [options] section
        let toml_no_options = r#"
            [input]
                read1 = "test.fq"
        "#;

        // Config with empty [options] section
        let toml_empty_options = r#"
            [input]
                read1 = "test.fq"
            [options]
        "#;

        let config_no_options: crate::config::Config = toml::from_str(toml_no_options).unwrap();
        let config_empty_options: crate::config::Config =
            toml::from_str(toml_empty_options).unwrap();

        // Both should have the same thread_count
        assert_eq!(
            config_no_options.options.thread_count, config_empty_options.options.thread_count,
            "thread_count should be the same whether [options] is missing or empty"
        );

        // Check all other fields too
        assert_eq!(
            config_no_options.options.block_size,
            config_empty_options.options.block_size
        );
        assert_eq!(
            config_no_options.options.buffer_size,
            config_empty_options.options.buffer_size
        );
        assert_eq!(
            config_no_options.options.output_buffer_size,
            config_empty_options.options.output_buffer_size
        );
        assert_eq!(
            config_no_options.options.accept_duplicate_files,
            config_empty_options.options.accept_duplicate_files
        );
        assert_eq!(
            config_no_options.options.spot_check_read_pairing,
            config_empty_options.options.spot_check_read_pairing
        );
    }

    #[test]
    fn test_default_consistency() {
        // Verify that Options::default() uses the same values as the field-level defaults
        let default_options = Options::default();

        assert_eq!(default_options.block_size, default_block_size());
        assert_eq!(default_options.buffer_size, default_buffer_size());
        assert_eq!(
            default_options.output_buffer_size,
            default_output_buffer_size()
        );
        assert!(!default_options.accept_duplicate_files);
        assert_eq!(
            default_options.spot_check_read_pairing,
            default_spot_check_read_pairing()
        );
    }
}
