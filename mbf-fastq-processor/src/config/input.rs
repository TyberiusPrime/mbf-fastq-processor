use std::collections::{BTreeMap, HashSet};

use anyhow::{Result, bail};
use indexmap::IndexMap;
use schemars::JsonSchema;
use std::collections::HashMap;
use toml_pretty_deser::prelude::*;

use crate::config::deser::tpd_extract_u8_from_byte_or_char;

use super::deser::{self, deserialize_map_of_string_or_seq_string};
use super::validate_segment_label;

fn is_default(opt: &InputOptions) -> bool {
    opt.fasta_fake_quality.is_none()
        && opt.bam_include_mapped.is_none()
        && opt.bam_include_unmapped.is_none()
        && opt.read_comment_character == deser::default_comment_insert_char()
}

pub const STDIN_MAGIC_PATH: &str = "--stdin--";

/// Input configuration
#[derive(serde::Serialize)]
#[tpd(partial = false)]
#[derive(Debug, Clone, JsonSchema)]
pub struct Input {
    /// whether you have input files with interleaved reads, or one file per segment
    /// If interleaved, define the name of the segments here.
    #[tpd_default]
    interleaved: Option<Vec<String>>,

    /// Your segments. Define just one with any name for interlaveed input.
    #[schemars(with = "BTreeMap<String, Vec<String>>")]
    #[tpd_absorb_remaining]
    #[serde(flatten)]
    segments: IndexMap<String, Vec<String>>,

    #[tpd_default_in_verify]
    #[tpd_nested]
    #[serde(skip_serializing_if = "is_default")]
    pub options: InputOptions,

    #[tpd_skip]
    #[schemars(skip)]
    #[serde(skip_serializing)]
    pub structured: Option<StructuredInput>,
    #[tpd_skip]
    #[serde(skip_serializing)]
    pub stdin_stream: bool,
}

impl VerifyFromToml for PartialInput {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.options = self.options.or_default_with(|| {
            //this could be prettier...
            let default = InputOptions::default();
            PartialInputOptions {
                fasta_fake_quality: TomlValue::new_ok(default.fasta_fake_quality, 0..0),
                bam_include_mapped: TomlValue::new_ok(default.bam_include_mapped, 0..0),
                bam_include_unmapped: TomlValue::new_ok(default.bam_include_unmapped, 0..0),
                read_comment_character: TomlValue::new_ok(default.read_comment_character, 0..0),
                use_rapidgzip: TomlValue::new_ok(default.use_rapidgzip, 0..0),
                build_rapidgzip_index: TomlValue::new_ok(default.build_rapidgzip_index, 0..0),
                threads_per_segment: TomlValue::new_ok(default.threads_per_segment, 0..0),
            }
        });
        self
    }
}

#[derive(serde::Serialize, Clone, PartialEq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct InputOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    //TODO #[serde(deserialize_with = "deser::opt_u8_from_char_or_number")]
    // #[validate(minimum = 33)]
    // #[validate(maximum = 126)]
    pub fasta_fake_quality: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_mapped: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bam_include_unmapped: Option<bool>,

    //Todo: #[serde(deserialize_with = "deser::u8_from_char_or_number")]
    #[tpd_adapt_in_verify]
    pub read_comment_character: u8,

    #[serde(skip_serializing)]
    pub use_rapidgzip: Option<bool>,

    #[serde(skip_serializing)]
    pub build_rapidgzip_index: Option<bool>,

    pub threads_per_segment: Option<usize>,
}

impl VerifyFromToml for PartialInputOptions {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.read_comment_character = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_read_comment_character(helper, false, false),
            self.tpd_get_read_comment_character(helper, false, false),)
            .or_default_with(deser::default_comment_insert_char);
        self
    }
}

impl Default for InputOptions {
    fn default() -> Self {
        InputOptions {
            fasta_fake_quality: None,
            bam_include_mapped: None,
            bam_include_unmapped: None,
            read_comment_character: deser::default_comment_insert_char(),
            use_rapidgzip: None,
            build_rapidgzip_index: None,
            threads_per_segment: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StructuredInput {
    Interleaved {
        files: Vec<String>,
        segment_order: Vec<String>,
    },
    Segmented {
        segment_files: IndexMap<String, Vec<String>>,
        segment_order: Vec<String>,
    },
}

impl Input {
    #[must_use]
    pub fn is_interleaved(&self) -> bool {
        self.interleaved.is_some()
    }

    #[must_use]
    pub fn segment_count(&self) -> usize {
        match self
            .structured
            .as_ref()
            .expect("structured input must be set after config parsing")
        {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    #[must_use]
    #[mutants::skip] // only used to figure out thread count.
    pub fn parser_count(&self) -> usize {
        match self
            .structured
            .as_ref()
            .expect("Called to early, structured not yet ready")
        {
            StructuredInput::Interleaved { .. } => 1,
            StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    #[must_use]
    pub fn get_segment_order(&self) -> &Vec<String> {
        match self
            .structured
            .as_ref()
            .expect("structured input must be set after config parsing")
        {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order,
        }
    }

    #[must_use]
    pub fn index(&self, segment_name: &str) -> Option<usize> {
        match self
            .structured
            .as_ref()
            .expect("structured input must be set after config parsing")
        {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => {
                segment_order.iter().position(|s| s == segment_name)
            }
        }
    }

    pub fn init(&mut self) -> Result<()> {
        if let Some(fake_fasta_quality) = self.options.fasta_fake_quality {
            if fake_fasta_quality < 33 || fake_fasta_quality > 126 {
                bail!(
                    "(input.options): fasta_fake_quality must be in the range [33..126]. Found: {}",
                    fake_fasta_quality
                );
            }
        }
        // Validate index_gzip option
        if let Some(true) = self.options.build_rapidgzip_index
            && !self.options.use_rapidgzip.unwrap_or_default()
        {
            bail!(
                "(input.options): build_rapidgzip_index=true is only valid when use_rapidgzip is set. Either unset build_rapidgzip_index or set use_rapidgzip=true ",
            );
        }

        //first me make sure all segments have the same number of files
        let no_of_file_per_segment: BTreeMap<_, _> =
            self.segments.iter().map(|(k, v)| (k, v.len())).collect();
        let observed_no_of_segments: HashSet<_> = no_of_file_per_segment.values().collect();
        if observed_no_of_segments.len() > 1 {
            let details: Vec<String> = no_of_file_per_segment
                .iter()
                .map(|(k, v)| format!("\t'{k}': \t{v}"))
                .collect();
            bail!(
                "(input): Number of files per segment is inconsistent:\n {}.\nEach segment must have the same number of files.",
                details.join(",\n")
            );
        }

        if let Some(interleaved) = &self.interleaved {
            if self.segments.len() != 1 {
                bail!(
                    "(input): Interleaved input can only have one other key defining the segments. Found: {} keys",
                    self.segments.len()
                );
            }
            if interleaved.len() < 2 {
                bail!(
                    "(input): Interleaved input must define at least two segments. Found: {}",
                    interleaved.len()
                );
            }
            self.structured = Some(StructuredInput::Interleaved {
                files: self
                    .segments
                    .values()
                    .next()
                    .cloned()
                    .expect("segmented input must have at least one segment"),
                segment_order: interleaved.iter().map(|x| x.trim().to_string()).collect(),
            });
        } else {
            let mut segment_order: Vec<String> =
                self.segments.keys().map(|x| x.trim().to_string()).collect();
            segment_order.sort(); //always alphabetical...
            if segment_order.is_empty() {
                bail!(
                    "(input): No segments defined in input. At least one ('read1' perhaps?) must be defined. Example: read1 = 'filename.fq'"
                );
            }
            if segment_order.iter().any(|x| x == "all" || x == "All") {
                bail!(
                    "(input): Segment name 'all' (or 'All') is reserved and cannot be used as a segment name."
                )
            }
            if segment_order
                .iter()
                .any(|x| x.eq_ignore_ascii_case("options"))
            {
                bail!(
                    "(input): Segment name 'options' (any case) is reserved and cannot be used as a segment name."
                );
            }
            if segment_order.iter().any(|x| x.starts_with("_internal_")) {
                bail!(
                    "(input): Segment names starting with '_internal_' are reserved and cannot be used as a segment name."
                )
            }

            self.structured = Some(StructuredInput::Segmented {
                segment_files: self.segments.clone(),
                segment_order,
            });
        }

        match self
            .structured
            .as_ref()
            .expect("structured input must be set after config parsing")
        {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => {
                let mut seen = HashSet::new();
                for key in segment_order {
                    if let Err(e) = validate_segment_label(key) {
                        bail!("(input): Invalid segment label '{key}': {e}");
                    }
                    /* if key.chars().any(|c| !(c.is_ascii())) {
                        bail!("Segment name may not contain non-ascii character");
                    } */

                    if !seen.insert(key) {
                        bail!("(input): Segment name duplicated: '{key}'")
                    }
                }
            }
        }
        self.validate_stdin_usage()?;
        Ok(())
    }

    fn validate_stdin_usage(&self) -> Result<()> {
        let Some(structured) = self.structured.as_ref() else {
            return Ok(());
        };
        match structured {
            StructuredInput::Interleaved { files, .. } => {
                if files.iter().any(|f| f == STDIN_MAGIC_PATH) {
                    if files.len() != 1 {
                        bail!(
                            "(input): Interleaved inputs may only use '{STDIN_MAGIC_PATH}' when exactly one input file is listed. Found {} files.",
                            files.len()
                        );
                    }
                    return Ok(());
                }
                Ok(())
            }
            StructuredInput::Segmented {
                segment_files,
                segment_order,
            } => {
                let segments_with_stdin: Vec<_> = segment_order
                    .iter()
                    .filter(|segment| {
                        segment_files
                            .get(*segment)
                            .is_some_and(|files| files.iter().any(|name| name == STDIN_MAGIC_PATH))
                    })
                    .collect();
                if segments_with_stdin.is_empty() {
                    return Ok(());
                }
                if segments_with_stdin.len() > 1 {
                    bail!(
                        "(input): '{STDIN_MAGIC_PATH}' may only appear in a single segment. Found it in segments: {segments_with_stdin:?}."
                    );
                }
                if segment_order.len() != 1 {
                    bail!(
                        "(input): Using '{STDIN_MAGIC_PATH}' requires exactly one segment. Defined segments: {segment_order:?}."
                    );
                }
                let segment = segments_with_stdin[0];
                let files = segment_files.get(segment).expect("segment must exist");
                if files.len() != 1 {
                    bail!(
                        "(input): Segment '{segment}' lists {} files. '{STDIN_MAGIC_PATH}' requires exactly one input file.",
                        files.len()
                    );
                }
                if files[0] != STDIN_MAGIC_PATH {
                    // Only possible if '--stdin--' was not first, but guard regardless.
                    bail!(
                        "(input): Segment '{segment}' mixes '{STDIN_MAGIC_PATH}' with additional paths. This magic value must be the only file in the segment."
                    );
                }
                Ok(())
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, JsonSchema)]
#[tpd]
pub enum CompressionFormat {
    #[tpd_alias("uncompressed")]
    #[tpd_alias("raw")]
    #[default]
    Uncompressed,
    #[tpd_alias("gzip")]
    #[tpd_alias("gz")]
    Gzip,
    #[tpd_alias("zstd")]
    #[tpd_alias("zst")]
    Zstd,
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

/// Validates that the compression level is within the expected range for the given compression format
pub fn validate_compression_level_u8(
    compression: CompressionFormat,
    compression_level: Option<u8>,
) -> Result<()> {
    if let Some(level) = compression_level {
        match compression {
            CompressionFormat::Uncompressed => {
                if level != 0 {
                    bail!(
                        "Compression level {level} specified for uncompressed output, but no compression is applied.",
                    );
                }
            }
            CompressionFormat::Gzip => {
                if level > 9 {
                    bail!(
                        "Compression level {level} is invalid for gzip compression. Valid range is 0-9.",
                    );
                }
            }
            CompressionFormat::Zstd => {
                if level == 0 || level > 22 {
                    bail!(
                        "Compression level {level} is invalid for zstd compression. Valid range is 1-22.",
                    );
                }
            }
        }
    } else {
        // No compression level specified - rapidgzip is still invalid for output
    }
    Ok(())
}
