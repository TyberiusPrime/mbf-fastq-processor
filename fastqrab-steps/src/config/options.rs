use anyhow::{Context, Result};
use schemars::JsonSchema;
use toml_pretty_deser::prelude::*;

use crate::config::{PartialConfig, StructuredInput};
use fastqrab_config::{
    default_block_size, default_buffer_size, default_output_buffer_size, default_reads_in_flight,
    default_spot_check_read_pairing,
};
use fastqrab_io::io::output::simulated_failure::{SimulatedWriteError, SimulatedWriteFailure};

#[derive(Clone, Default, JsonSchema)]
#[tpd(no_verify)]
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
            FailOutputError::RawOs => {
                let code = self
                    .fail_output_raw_os_code
                    .context(
                        "options.debug_failures.fail_output_raw_os_code required when fail_output_error = 'raw_os'",
                    )?; // cov:excl-line
                SimulatedWriteError::RawOs(code)
            }
        };

        Ok(Some(SimulatedWriteFailure {
            remaining_bytes,
            error,
        }))
    }
}

#[tpd]
#[derive(Debug, Clone, JsonSchema)]
pub enum FailOutputError {
    DiskFull,
    RawOs,
}

#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Options {
    #[tpd(alias = "thread_count")]
    pub threads: Option<usize>,
    #[schemars(with = "Option<usize>")]
    pub max_reads_in_flight: usize,

    #[schemars(with = "Option<usize>")]
    pub block_size: usize,
    #[schemars(with = "Option<usize>")]
    pub buffer_size: usize,
    #[schemars(with = "Option<usize>")]
    pub output_buffer_size: usize,
    #[tpd(default)]
    #[schemars(with = "Option<bool>")]
    pub accept_duplicate_files: bool,
    #[schemars(with = "Option<bool>")]
    pub spot_check_read_pairing: bool,
    #[tpd(nested)]
    #[schemars(skip)]
    pub debug_failures: FailureOptions,
}

impl VerifyIn<PartialConfig> for PartialOptions {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.block_size.or_with(|| default_block_size().into());
        self.max_reads_in_flight.or_with(|| default_reads_in_flight(
            *self.block_size.as_ref().expect("Just defaulted"),
        ));
        self.buffer_size.or_with(default_buffer_size);
        self.output_buffer_size.or_with(default_output_buffer_size);
        self.accept_duplicate_files.or(false);
        self.spot_check_read_pairing
            .or_with(default_spot_check_read_pairing);
        self.debug_failures.or_with(|| PartialFailureOptions {
            fail_output_after_bytes: TomlValue::new_ok(None, 0..0),
            fail_output_error: TomlValue::new_ok(None, 0..0),
            fail_output_raw_os_code: TomlValue::new_ok(None, 0..0),
        });

        self.block_size.verify(|v| {
            if *v == 0 {
                return Err(ValidationFailure::new(
                    "Must be > 0",
                    Some("Set to a positive integer."),
                ));
            }
            if parent
                .input
                .as_ref()
                .and_then(|input_def| input_def.structured.as_ref())
                .is_some_and(StructuredInput::is_interleaved)
                && !v.is_multiple_of(2)
            {
                return Err(ValidationFailure::new(
                    "block_size must be a multiple of 2",
                    Some("Either set an block_size, or remove interleaved"),
                ));
            }

            Ok(())
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    use super::*;

    #[test]
    fn test_options_deserialize_missing_vs_empty() {
        // Test that missing [options] section and empty [options] section
        // produce the same result

        // Config with no [options] section
        let toml_no_options = r#"
            [input]
                read1 = "test.fq"
            [output]
                prefix = 'out'
            [[step]]
                action = 'OutputFASTQ'
        "#;

        // Config with empty [options] section
        let toml_empty_options = r#"
            [input]
                read1 = "test.fq"
            [options]
            [output]
                prefix = 'out'

            [[step]]
                action = 'OutputFASTQ'
        "#;

        let config_no_options =
            Config::tpd_from_toml(toml_no_options, FieldMatchMode::AnyCase, VecMode::SingleOk);

        dbg!(&config_no_options);
        let config_no_options = config_no_options.unwrap();
        let config_empty_options = Config::tpd_from_toml(
            toml_empty_options,
            FieldMatchMode::AnyCase,
            VecMode::SingleOk,
        );
        let config_empty_options = config_empty_options.unwrap();

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
}
