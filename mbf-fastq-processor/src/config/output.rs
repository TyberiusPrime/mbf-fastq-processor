use schemars::JsonSchema;

use crate::config::deser::{
     ErrorCollectorExt, FromTomlTable, TomlResult,
};

use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub prefix: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: FileFormat,
    #[serde(default)]
    pub compression: CompressionFormat,
    #[serde(default)]
    pub compression_level: Option<u8>,
    #[serde(default)]
    pub compression_threads: Option<usize>,

    #[serde(default)]
    pub report_html: bool,
    #[serde(default)]
    pub report_json: bool,
    #[serde(default)]
    pub report_timing: bool,

    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub interleave: Option<Vec<String>>,

    #[serde(default)]
    pub output: Option<Vec<String>>,

    #[serde(default)]
    pub output_hash_uncompressed: bool,
    #[serde(default)]
    pub output_hash_compressed: bool,
    #[serde(default = "default_ix_separator")]
    pub ix_separator: String,

    #[serde(default)]
    #[serde(rename = "Chunksize")]
    #[serde(alias = "chunk_size")]
    #[serde(alias = "chunksize")]
    pub chunksize: Option<usize>,
}

impl FromTomlTable for Output {
    fn from_toml_table(
        table: &toml_edit::Table,
        collector: &crate::config::deser::ErrorCollector,
    ) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut helper = collector.local(table);

        let prefix = helper.get("prefix");
        let suffix = helper.get_opt("suffix");
        let format = helper.get_opt::<FileFormat>("format");
        let compression = helper.get_opt::<CompressionFormat>("compression");
        let compression_level = helper.get_opt("compression_level");
        let compression_threads = helper.get_opt("compression_threads");
        let report_html = helper.get_opt("report_html");
        let report_json = helper.get_opt("report_json");
        let report_timing = helper.get_opt("report_timing");
        let stdout = helper.get_opt("stdout");
        let interleave = helper.get_opt("interleave");
        let output = helper.get_opt("output");
        let output_hash_uncompressed = helper.get_opt("output_hash_uncompressed");
        let output_hash_compressed = helper.get_opt("output_hash_compressed");
        let ix_separator = helper.get_opt("ix_separator");
        let chunksize = helper.get_opt("chunksize");

        helper.deny_unknown()?;

        Ok(Output {
            prefix: prefix?,
            suffix: suffix?,
            format: format?.unwrap_or_default(), //todo
            compression: compression?.unwrap_or_default(),
            compression_level: compression_level?,
            compression_threads: compression_threads?,
            report_html: report_html?.unwrap_or(false),
            report_json: report_json?.unwrap_or(false),
            report_timing: report_timing?.unwrap_or(false),
            stdout: stdout?.unwrap_or(false),
            interleave: interleave?,
            output: output?,
            output_hash_uncompressed: output_hash_uncompressed?.unwrap_or(false),
            output_hash_compressed: output_hash_compressed?.unwrap_or(false),
            ix_separator: ix_separator?.unwrap_or_else(default_ix_separator),
            chunksize: chunksize?,
        })
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
