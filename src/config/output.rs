use super::{CompressionFormat, FileFormat};

#[must_use]
pub fn default_ix_separator() -> String {
    "_".to_string()
}

#[derive(eserde::Deserialize, Debug, Clone)]
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
    pub report_html: bool,
    #[serde(default)]
    pub report_json: bool,

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
}

impl Output {
    #[must_use]
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}
