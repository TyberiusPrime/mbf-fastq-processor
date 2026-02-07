use schemars::JsonSchema;
use toml_pretty_deser::prelude::*;

use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(Clone, JsonSchema)]
#[tpd(partial = false)]
#[derive(Debug)]
pub struct Output {
    pub prefix: String,
    #[tpd_default]
    pub suffix: Option<String>,
    #[tpd_default]
    pub format: FileFormat,
    #[tpd_default]
    pub compression: CompressionFormat,
    #[tpd_default]
    pub compression_level: Option<u8>,
    #[tpd_default]
    pub compression_threads: Option<usize>,

    #[tpd_default]
    pub report_html: bool,
    #[tpd_default]
    pub report_json: bool,
    #[tpd_default]
    pub report_timing: bool,

    #[tpd_default]
    pub stdout: bool,
    #[tpd_default]
    pub interleave: Option<Vec<String>>,

    #[tpd_default]
    pub output: Option<Vec<String>>,

    #[tpd_default]
    pub output_hash_uncompressed: bool,
    #[tpd_default]
    pub output_hash_compressed: bool,
    #[tpd_default_in_verify]
    pub ix_separator: String,

    #[tpd_default]
    pub chunksize: Option<usize>,
}

impl VerifyFromToml for PartialOutput {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.ix_separator = self.ix_separator.or_default_with(default_ix_separator);
        self
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
