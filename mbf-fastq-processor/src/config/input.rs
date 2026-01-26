use std::collections::{BTreeMap, HashSet};

use anyhow::{Result, bail};
use schemars::JsonSchema;

use crate::config::deser::{
    ErrorCollector, ErrorCollectorExt, FromToml, FromTomlTable, TomlResult, TomlResultExt,
};

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
#[derive(eserde::Deserialize, Debug, Clone, serde::Serialize, JsonSchema)]
pub struct Input {
    /// whether you have input files with interleaved reads, or one file per segment
    /// If interleaved, define the name of the segments here.
    #[serde(default)]
    interleaved: Option<Vec<String>>,

    /// Your segments. Define just one with any name for interlavede input.
    #[serde(flatten, deserialize_with = "deserialize_map_of_string_or_seq_string")]
    segments: BTreeMap<String, Vec<String>>,

    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub options: InputOptions,

    // Computed field for consistent ordering - not serialized
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub structured: Option<StructuredInput>, //todo: remove option
    #[serde(default)]
    #[serde(skip)]
    pub stdin_stream: bool,
}
//first me make sure all segments have the same number of files
fn construct_structured_input(
    segments: &BTreeMap<String, Vec<String>>,
    interleaved: &Option<Vec<String>>,
) -> Result<StructuredInput> {
    let no_of_file_per_segment: BTreeMap<_, _> =
        segments.iter().map(|(k, v)| (k, v.len())).collect();
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

    let structured = if let Some(interleaved) = interleaved {
        if segments.len() != 1 {
            bail!(
                "(input): Interleaved input can only have one other key defining the segments. Found: {} keys",
                segments.len()
            );
        }
        if interleaved.len() < 2 {
            bail!(
                "(input): Interleaved input must define at least two segments. Found: {}",
                interleaved.len()
            );
        }
        StructuredInput::Interleaved {
            files: segments
                .values()
                .next()
                .cloned()
                .expect("segmented input must have at least one segment"),
            segment_order: interleaved.iter().map(|x| x.trim().to_string()).collect(),
        }
    } else {
        let mut segment_order: Vec<String> =
            segments.keys().map(|x| x.trim().to_string()).collect();
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

        StructuredInput::Segmented {
            segment_files: segments.clone(),
            segment_order,
        }
    };

    match &structured {
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
    validate_stdin_usage(&structured)?;
    Ok(structured)
}

fn validate_stdin_usage(structured: &StructuredInput) -> Result<()> {
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
            match segments_with_stdin.len() {
                0 => Ok(()),
                1 => {
                    let segment = segments_with_stdin[0];
                    let files = segment_files.get(segment).expect("segment must exist");
                    if files.len() != 1 {
                        bail!(
                            "(input): Segment '{segment}' lists {} files. '{STDIN_MAGIC_PATH}' requires exactly one input file.",
                            files.len()
                        );
                    }
                    // if files[0] != STDIN_MAGIC_PATH {
                    //     // Only possible if '--stdin--' was not first, but guard regardless.
                    //     bail!(
                    //         "(input): Segment '{segment}' mixes '{STDIN_MAGIC_PATH}' with additional paths. This magic value must be the only file in the segment."
                    //     );
                    // }
                    Ok(())
                }
                _ => bail!(
                    "(input): '{STDIN_MAGIC_PATH}' may only appear in a single segment. Found it in segments: {segments_with_stdin:?}."
                ),
            }
        }
    }
}

impl FromTomlTable for Input {
    fn from_toml_table(table: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let res = {
            let mut helper = collector.local(table);
            let segments: TomlResult<BTreeMap<String, Vec<String>>> = helper
                .table
                .iter()
                .filter(|(k, _)| *k != "options" && *k != "interleaved")
                .map(|(k, v)| match v {
                    toml_edit::Item::Value(toml_edit::Value::String(s)) => {
                        Ok((k.to_string(), vec![s.value().to_string()]))
                    }
                    toml_edit::Item::Value(toml_edit::Value::Array(arr)) => {
                        let values: TomlResult<Vec<String>> = arr
                            .iter()
                            .map(|x| match x.as_str() {
                                Some(x) => Ok(x.to_string()),
                                None => collector.add_value(
                                    x,
                                    "Expected a string value",
                                    "Supply a filename.",
                                ),
                            })
                            .collect();
                        Ok((k.to_string(), values?))
                    }
                    _ => collector.add_item(
                        v,
                        "Must be a string or list of strings",
                        "Example: `read1 = 'input.fq'`",
                    ),
                })
                .collect();

            let interleaved = helper.get_opt("interleaved");
            let options = helper.get_opt("options");
            helper.accept_unknown()?; // we collected them into segments
            //
            let segments = segments?;
            let interleaved = interleaved?;
            let structured = construct_structured_input(&segments, &interleaved)
                .or_else(|e| collector.add_table(table, &e.to_string(), ""))?;

            Input {
                interleaved: interleaved,
                segments: segments,
                options: options?.unwrap_or_else(InputOptions::default),
                structured: Some(structured),
                stdin_stream: false,
            }
        };

        collector
            .borrow_mut()
            .set_segment_order(res.get_segment_order().clone());
        Ok(res)
    }
}

#[derive(eserde::Deserialize, Debug, Clone, serde::Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deser::opt_u8_from_char_or_number")]
    #[serde(default)]
    // #[validate(minimum = 33)]
    // #[validate(maximum = 126)]
    pub fasta_fake_quality: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bam_include_mapped: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bam_include_unmapped: Option<bool>,

    #[serde(deserialize_with = "deser::u8_from_char_or_number")]
    #[serde(default = "deser::default_comment_insert_char")]
    pub read_comment_character: u8,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub use_rapidgzip: Option<bool>,

    #[serde(skip_serializing)]
    #[serde(default)]
    pub build_rapidgzip_index: Option<bool>,

    #[serde(default)]
    pub threads_per_segment: Option<usize>,
}
//first me make sure all segments have the same number of files
impl FromTomlTable for InputOptions {
    fn from_toml_table(table: &toml_edit::Table, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let mut helper = collector.local(table);
        let fasta_fake_quality = helper
            .get_opt_u8_from_char_or_number("fasta_fake_quality", Some(33), Some(126))
            .add_help("Must be a valid PHRED value");
        let bam_include_mapped = helper.get_opt("bam_include_mapped");
        let bam_include_unmapped = helper.get_opt("bam_include_unmapped");

        let read_comment_character =
            helper.get_opt_u8_from_char_or_number("read_comment_character", Some(33), Some(126));
        let use_rapidgzip: TomlResult<Option<bool>> = helper.get_opt("use_rapidgzip");
        let build_rapidgzip_index = helper.get_opt("build_rapidgzip_index");
        let threads_per_segment = helper.get_opt_clamped("threads_per_segment", None, None);

        if let Ok(Some(true)) = build_rapidgzip_index
            && let Ok(Some(false)) = use_rapidgzip
        {
            helper
                .add_err_by_key::<()>(
                    "build_rapidgzip_index",
                    "build_rapidgzip_index=true is only valid when use_rapidgzip is set.",
                    "Either unset build_rapidgzip_index or set use_rapidgzip=true ",
                )
                .ok(); //implicitly sets error_occured which will then Err in helper.deny_unknown() 
        }
        helper.deny_unknown()?;
        Ok(InputOptions {
            fasta_fake_quality: fasta_fake_quality?,
            bam_include_mapped: bam_include_mapped?,
            bam_include_unmapped: bam_include_unmapped?,
            read_comment_character: read_comment_character?
                .unwrap_or_else(deser::default_comment_insert_char),
            use_rapidgzip: use_rapidgzip?,
            build_rapidgzip_index: build_rapidgzip_index?,
            threads_per_segment: threads_per_segment?,
        })
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
        segment_files: BTreeMap<String, Vec<String>>,
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
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, eserde::Deserialize, JsonSchema)]
pub enum CompressionFormat {
    #[serde(alias = "uncompressed")]
    #[serde(alias = "Uncompressed")]
    #[serde(alias = "raw")]
    #[serde(alias = "Raw")]
    #[default]
    Uncompressed,
    #[serde(alias = "gzip")]
    #[serde(alias = "gz")]
    #[serde(alias = "Gzip")]
    #[serde(alias = "Gz")]
    Gzip,
    #[serde(alias = "zstd")]
    #[serde(alias = "zst")]
    #[serde(alias = "Zstd")]
    #[serde(alias = "Zst")]
    Zstd,
}

impl FromToml for CompressionFormat {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        if let Some(v) = value.as_str() {
            match v.to_lowercase().as_str() {
                "uncompressed" | "raw" => return Ok(CompressionFormat::Uncompressed),
                "gzip" | "gz" => return Ok(CompressionFormat::Gzip),
                "zstd" | "zst" => return Ok(CompressionFormat::Zstd),
                _ => {}
            }
        }
        collector.invalid_value(value, &["Gzip", "Zstd", "Uncompressed"])
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, eserde::Deserialize, JsonSchema)]
pub enum FileFormat {
    #[serde(alias = "fastq")]
    #[serde(alias = "FastQ")]
    #[serde(alias = "Fastq")]
    #[serde(alias = "FASTQ")]
    #[default]
    Fastq,
    #[serde(alias = "fasta")]
    #[serde(alias = "Fasta")]
    #[serde(alias = "FASTA")]
    Fasta,
    #[serde(alias = "bam")]
    #[serde(alias = "Bam")]
    #[serde(alias = "BAM")]
    Bam,
    #[serde(alias = "none")]
    #[serde(alias = "None")]
    None,
}

impl FromToml for FileFormat {
    fn from_toml(value: &toml_edit::Item, collector: &ErrorCollector) -> TomlResult<Self>
    where
        Self: Sized,
    {
        if let Some(v) = value.as_str() {
            match &v.to_lowercase()[..] {
                "fastq" => return Ok(FileFormat::Fastq),
                "fasta" => return Ok(FileFormat::Fasta),
                "bam" => return Ok(FileFormat::Bam),
                "none" => return Ok(FileFormat::None),
                _ => { //fallthrouh
                }
            }
        }
        collector.invalid_value(value, &["FASTQ", "FASTA", "BAM", "None"])
    }
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
