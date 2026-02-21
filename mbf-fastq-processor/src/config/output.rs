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
    fn verify(&mut self, parent: &super::PartialConfig) -> Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.ix_separator.or_with(default_ix_separator);
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
