#![allow(clippy::struct_field_names)] // FailureOptions - eserde(?) interferes with clippy here. 
use crate::{
    config::deser::{
        ErrorCollector, ErrorCollectorExt, FromToml, FromTomlTable, TomlResult,
        TomlResultExt
    },
    io::output::compressed_output::{SimulatedWriteError, SimulatedWriteFailure},
};
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

impl FromTomlTable for FailureOptions {
    fn from_toml_table(table: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut local = collector.local(table);
        let fail_output_after_bytes =
            local.get_opt_clamped("fail_output_after_bytes", Some(1), None);
        let fail_output_error = local.get_opt("fail_output_error");
        let fail_output_raw_os_code = local.get_opt("fail_output_raw_os_code");
        local.deny_unknown()?;
        Ok(FailureOptions {
            fail_output_after_bytes: fail_output_after_bytes?,
            fail_output_error: fail_output_error?,
            fail_output_raw_os_code: fail_output_raw_os_code?,
        })
    }
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

#[derive(eserde::Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailOutputError {
    DiskFull,
    Other,
    RawOs,
}

impl FromToml for FailOutputError {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self> {
        collector.match_str(
            value,
            &[
                ("DiskFull", FailOutputError::DiskFull),
                ("Other", FailOutputError::Other),
                ("RawOs", FailOutputError::RawOs),
            ],
        )
    }
}

#[must_use]
#[mutants::skip]
pub fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

#[mutants::skip]
fn default_output_buffer_size() -> usize {
    1024 * 1024 // bytes, per fastq input file
}

#[must_use]
#[mutants::skip]
pub fn default_block_size() -> usize {
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

fn default_spot_check_read_pairing() -> bool {
    true
}

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default)]
    #[serde(alias = "thread_count")]
    pub threads: Option<usize>,
    #[serde(default)]
    pub max_blocks_in_flight: Option<usize>,

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

impl FromTomlTable for Options {
    fn from_toml_table(table: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut helper = collector.local(table);
        let threads = helper.get_opt("threads");
        let max_blocks_in_flight = helper.get_opt_clamped("max_blocks_in_flight", Some(1), None);
        let block_size = helper
            .get_opt_clamped("block_size", Some(1), None)
            .add_help("Supply a positive number, e.g. 10_000")
        ;
        let buffer_size = helper
            .get_opt_clamped("buffer_size", Some(1), None)
            ;
        let output_buffer_size = helper
            .get_opt_clamped("output_buffer_size", Some(1), None)
            ;
        let accept_duplicate_files = helper.get_opt("accept_duplicate_files");
        let spot_check_read_pairing = helper.get_opt("spot_check_read_pairing");
        let debug_failures = helper.get_opt("debug_failures");
        helper.deny_unknown()?;
        Ok(Self {
            threads: threads?,
            max_blocks_in_flight: max_blocks_in_flight?,
            block_size: block_size?.unwrap_or_else(default_block_size),
            buffer_size: buffer_size?.unwrap_or_else(default_buffer_size),
            output_buffer_size: output_buffer_size?
                .unwrap_or_else(default_output_buffer_size),
            accept_duplicate_files: accept_duplicate_files?.unwrap_or(false),
            spot_check_read_pairing: spot_check_read_pairing?.unwrap_or(true),
            debug_failures: debug_failures?.unwrap_or_default()
        })
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
