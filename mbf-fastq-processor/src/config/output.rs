use schemars::JsonSchema;

use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    /// Files get named {prefix}_{segment}{suffix}, e.g. 'output_read1.fq.gz'
    pub prefix: String,
    /// Optional, determined by the format if left off
    #[serde(default)]
    pub suffix: Option<String>,
    /// Output format: Fastq, Fasta, BAM, or None (default: Fastq)
    #[serde(default)]
    pub format: FileFormat,
    /// Gzip, Zstd, or uncompressed (default: uncompressed)
    #[serde(default)]
    pub compression: CompressionFormat,
    /// Compression level for gzip (0-9) or zstd (1-22). Defaults: gzip=6, zstd=5
    #[serde(default)]
    pub compression_level: Option<u8>,

    /// Write an interactive html report file ($prefix.html)?
    #[serde(default)]
    pub report_html: bool,
    /// Write a json report file ($prefix.json)?
    #[serde(default)]
    pub report_json: bool,
    /// Write timing statistics to json file ($prefix_timing.json)?
    #[serde(default)]
    pub report_timing: bool,

    /// Write read1 to stdout, do not produce other fastq files
    #[serde(default)]
    pub stdout: bool,
    /// Interleave fastq output, producing only a single output file for read1/read2
    #[serde(default)]
    pub interleave: Option<Vec<String>>,

    /// Which segments to write. Defaults to all segments defined in [input]
    #[serde(default)]
    pub output: Option<Vec<String>>,

    #[serde(default)]
    pub output_hash_uncompressed: bool,
    #[serde(default)]
    pub output_hash_compressed: bool,
    /// Separator inserted between prefix, infix, and segment names (default: '_')
    #[serde(default = "default_ix_separator")]
    pub ix_separator: String,

    /// Maximum number of molecules per output file
    #[serde(default)]
    #[serde(rename = "Chunksize")]
    #[serde(alias = "chunk_size")]
    #[serde(alias = "chunksize")]
    pub chunksize: Option<usize>,
}

impl Output {
    #[must_use]
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}
