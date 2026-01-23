use schemars::JsonSchema;

use crate::config::deser::{FromTomlTable, TableExt, TomlResult};

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
    fn from_toml_table(table: &toml_edit::Table) -> TomlResult<Self>
    where
        Self: Sized,
    {
        Ok(Output {
            prefix: table.getx("prefix")?,
            suffix: table.getx_opt("suffix")?,
            format: table.getx_opt::<FileFormat>("format")?.unwrap_or_default(),
            compression: table
                .getx_opt::<CompressionFormat>("compression")?
                .unwrap_or_default(),
            compression_level: table.getx_opt("compression_level")?,
            compression_threads: table.getx_opt("compression_threads")?,

            report_html: table.getx_opt("report_html")?.unwrap_or(false),
            report_json: table.getx_opt("report_json")?.unwrap_or(false),
            report_timing: table.getx_opt("report_timing")?.unwrap_or(false),

            stdout: table.getx_opt("stdout")?.unwrap_or(false),
            interleave: table.getx_opt("interleave")?,
            output: table.getx_opt("output")?,

            output_hash_uncompressed: table.getx_opt("output_hash_uncompressed")?.unwrap_or(false),
            output_hash_compressed: table.getx_opt("output_hash_compressed")?.unwrap_or(false),
            ix_separator: table
                .getx_opt("ix_separator")?
                .unwrap_or_else(default_ix_separator),

            chunksize: table.getx_opt("chunksize")?,
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
