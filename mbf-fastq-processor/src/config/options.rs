#![allow(clippy::struct_field_names)] // FailureOptions - eserde(?) interferes with clippy here. 
use crate::io::output::compressed_output::{SimulatedWriteError, SimulatedWriteFailure};
use anyhow::{Context, Result};
use schemars::JsonSchema;
use toml_pretty_deser::prelude::*;
use crate::config::input::PartialInput;

#[derive(Clone, Default, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct FailureOptions {
    pub fail_output_after_bytes: Option<usize>,
    pub fail_output_error: Option<FailOutputError>,
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

#[tpd]
#[derive(Debug, Clone, JsonSchema)]
pub enum FailOutputError {
    DiskFull,
    Other,
    RawOs,
}

#[must_use]
#[mutants::skip]
pub fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

#[mutants::skip]
const fn default_output_buffer_size() -> usize {
    1024 * 1024 // bytes, per fastq input file
}

#[must_use]
#[mutants::skip]
pub const fn default_block_size() -> usize {
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

const fn default_spot_check_read_pairing() -> bool {
    true
}

#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Options {
    //#[serde(default)]
    #[tpd(alias="thread_count")]
    pub threads: Option<usize>,
    pub max_blocks_in_flight: Option<usize>,

    #[tpd_default_in_verify]
    pub block_size: usize,
    #[tpd_default_in_verify]
    pub buffer_size: usize,
    #[tpd_default_in_verify]
    pub output_buffer_size: usize,
    #[tpd_default]
    pub accept_duplicate_files: bool,
    //#[serde(default = "default_spot_check_read_pairing")]
    #[tpd_default_in_verify]
    pub spot_check_read_pairing: bool,
    #[tpd(nested)]
    #[tpd_default]
    pub debug_failures: FailureOptions,
}

impl VerifyIn<PartialInput> for PartialOptions {
    fn verify(mut self, _helper: &mut TomlHelper<'_>, _parent: &PartialInput) -> Self
    where
        Self: Sized,
    {
        self.block_size = self.block_size.or_default(default_block_size());
        self.buffer_size = self.buffer_size.or_default(default_buffer_size());
        self.output_buffer_size = self
            .output_buffer_size
            .or_default(default_output_buffer_size());
        self.accept_duplicate_files = self.accept_duplicate_files.or_default(false);
        self.spot_check_read_pairing = self
            .spot_check_read_pairing
            .or_default(default_spot_check_read_pairing());

        self
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            threads: None,
            max_blocks_in_flight: None,
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
    use crate::config::{Config, PartialConfig};

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

        let config_no_options = deserialize_with_mode::<PartialConfig, Config>(
            &toml_no_options,
            FieldMatchMode::AnyCase,
            VecMode::SingleOk,
        );

        let config_no_options = config_no_options.unwrap();
        let config_empty_options = deserialize_with_mode::<PartialConfig, Config>(
            &toml_empty_options,
            FieldMatchMode::AnyCase,
            VecMode::SingleOk,
        )
        .unwrap();

        // Both should have the same threads
        assert_eq!(
            config_no_options.options.threads, config_empty_options.options.threads,
            "threads should be the same whether [options] is missing or empty"
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