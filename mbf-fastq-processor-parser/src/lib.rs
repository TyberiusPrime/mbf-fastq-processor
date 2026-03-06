pub mod io;
use schemars::JsonSchema;
use toml_pretty_deser::prelude::tpd;

pub const STDIN_MAGIC_PATH: &str = "--stdin--";

#[must_use]
#[mutants::skip]
pub fn get_number_of_cores() -> usize {
    std::env::var("MBF_FASTQ_PROCESSOR_NUM_CPUS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(num_cpus::get)
}



#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
#[tpd]
pub enum CompressionFormat {
    #[tpd(alias = "uncompressed")]
    #[tpd(alias = "raw")]
    #[default]
    Uncompressed,
    #[tpd(alias = "gzip")]
    #[tpd(alias = "gz")]
    Gzip,
    #[tpd(alias = "zstd")]
    #[tpd(alias = "zst")]
    Zstd,
}

impl CompressionFormat {
    #[must_use]
    pub fn is_compressed(&self) -> bool {
        !matches!(self, CompressionFormat::Uncompressed)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, JsonSchema)]
#[tpd]
pub enum FileFormat {
    #[default]
    Fastq,
    Fasta,
    Bam,
    None,
}

impl FileFormat {
    #[must_use]
    pub fn default_suffix(&self) -> &'static str {
        match self {
            FileFormat::Fastq => "fq",
            FileFormat::Fasta => "fasta",
            FileFormat::Bam => "bam",
            FileFormat::None => "",
        }
    }

    #[must_use]
    pub fn get_suffix(
        &self,
        compression: CompressionFormat,
        custom_suffix: Option<&String>,
    ) -> String {
        if let Some(custom) = custom_suffix {
            return custom.clone();
        }

        match self {
            FileFormat::Fastq | FileFormat::Fasta => {
                let base = self.default_suffix();
                compression.apply_suffix(base)
            }
            FileFormat::Bam => self.default_suffix().to_string(),
            FileFormat::None => String::new(),
        }
    }
}

impl CompressionFormat {
    #[must_use]
    pub fn apply_suffix(self, base: &str) -> String {
        match self {
            CompressionFormat::Uncompressed => base.to_string(),
            CompressionFormat::Gzip => format!("{base}.gz"),
            CompressionFormat::Zstd => format!("{base}.zst"),
        }
    }
}
