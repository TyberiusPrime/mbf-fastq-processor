use schemars::JsonSchema;
use toml_pretty_deser::prelude::*;

use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Output {
    pub prefix: String,
    #[tpd(default)]
    pub suffix: Option<String>,
    #[tpd(default)]
    pub format: FileFormat,
    #[tpd(default)]
    pub compression: CompressionFormat,
    #[tpd(default)]
    pub compression_level: Option<u8>,
    #[tpd(default)]
    pub compression_threads: Option<usize>,

    #[tpd(default)]
    pub report_html: bool,
    #[tpd(default)]
    pub report_json: bool,
    #[tpd(default)]
    pub report_timing: bool,

    #[tpd(default)]
    pub stdout: bool,
    #[tpd(default)]
    pub interleave: Option<Vec<String>>,

    #[tpd(default)]
    pub output: Option<Vec<String>>,

    #[tpd(default)]
    pub output_hash_uncompressed: bool,
    #[tpd(default)]
    pub output_hash_compressed: bool,
    pub ix_separator: String,

    #[tpd(default)]
    pub chunksize: Option<usize>,
}

impl VerifyIn<super::PartialConfig> for PartialOutput {
    fn verify(&mut self, _parent: &super::PartialConfig) -> Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.ix_separator.or_with(default_ix_separator);
        if let Some(Some(level)) = self.compression_level.value {
            if !self
                .compression
                .as_ref()
                .map(|c| c.is_compressed())
                .unwrap_or(false)
            {
                self.compression_level.state = TomlValueState::ValidationFailed {
                    message: "Invalid when compression='uncompressed'".to_string(),
                };
                self.compression_level.help = Some(
                "Either remove the compression_lever parameter, or set the compression to 'Gzip'/'Zstd'".to_string());
            } else {
                validate_compression_level_u8(&mut self.compression, &mut self.compression_level);
            }
        }
        Ok(())
    }
}

impl Output {
    #[must_use]
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            suffix: None,
            format: FileFormat::Fastq,
            compression: CompressionFormat::Uncompressed,
            compression_level: None,
            compression_threads: None,
            report_html: false,
            report_json: false,
            report_timing: false,
            stdout: false,
            interleave: None,
            output: None,
            output_hash_uncompressed: false,
            output_hash_compressed: false,
            ix_separator: default_ix_separator(),
            chunksize: None,
        }
    }

    #[must_use]
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}

/// Validates that the compression level is within the expected range for the given compression format
pub fn validate_compression_level_u8(
    compression: &TomlValue<CompressionFormat>,
    compression_level: &mut TomlValue<Option<u8>>,
) {
    if let Some(Some(level)) = compression_level.as_ref() {
        match compression.as_ref() {
            Some(CompressionFormat::Uncompressed) => {
                if *level != 0 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: "Compression level {level} specified for uncompressed output"
                            .to_string(),
                    };
                    compression_level.help = Some("Remove compression_level".to_string());
                }
            }
            Some(CompressionFormat::Gzip) => {
                if *level > 9 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: "Invalid Value".to_string(),
                    };
                    compression_level.help = Some("Valid range is 0-9 for gzip.".to_string());
                }
            }
            Some(CompressionFormat::Zstd) => {
                if *level == 0 || *level > 22 {
                    compression_level.state = TomlValueState::ValidationFailed {
                        message: ("Invalid Value".to_string()),
                    };
                    compression_level.help = Some("Valid range is 1-22 for zstd.".to_string());
                }
            }
            None => {
                //nothing to verify, compression not set
            }
        }
    } else {
        // No compression level specified - rapidgzip is still invalid for output
    }
}
