#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
use crate::io::{self, DetectedInputFormat};
use crate::output::{SimulatedWriteError, SimulatedWriteFailure};
use crate::transformations::{Step, TagValueType, Transformation};
use anyhow::{Context, Result, anyhow, bail};
use bstr::BString;
use serde_valid::Validate;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub mod deser;

use deser::deserialize_map_of_string_or_seq_string;

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[derive(eserde::Deserialize, Debug, Clone, serde::Serialize)]
pub struct Input {
    #[serde(default)]
    interleaved: Option<Vec<String>>,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default")]
    pub options: InputOptions,
    #[serde(flatten, deserialize_with = "deserialize_map_of_string_or_seq_string")]
    segments: HashMap<String, Vec<String>>,

    // Computed field for consistent ordering - not serialized
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub structured: Option<StructuredInput>,
}

#[derive(eserde::Deserialize, Debug, Clone, Default, serde::Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InputOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deser::opt_u8_from_char_or_number")]
    #[serde(default)]
    pub fasta_fake_quality: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bam_include_mapped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub bam_include_unmapped: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum StructuredInput {
    Interleaved {
        files: Vec<String>,
        segment_order: Vec<String>,
    },
    Segmented {
        segment_files: HashMap<String, Vec<String>>,
        segment_order: Vec<String>,
    },
}

impl Input {
    #[must_use]
    pub fn segment_count(&self) -> usize {
        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    #[must_use]
    pub fn get_segment_order(&self) -> &Vec<String> {
        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order,
        }
    }

    #[must_use]
    pub fn index(&self, segment_name: &str) -> Option<usize> {
        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => {
                segment_order.iter().position(|s| s == segment_name)
            }
        }
    }

    fn init(&mut self) -> Result<()> {
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
                "Number of files per segment is inconsistent:\n {}.\nEach segment must have the same number of files.",
                details.join(",\n")
            );
        }

        if let Some(interleaved) = &self.interleaved {
            if self.segments.len() != 1 {
                bail!(
                    "Interleaved input can only have one other key defining the segments. Found: {} keys",
                    self.segments.len()
                );
            }
            if interleaved.len() < 2 {
                bail!(
                    "Interleaved input must define at least two segments. Found: {}",
                    interleaved.len()
                );
            }
            self.structured = Some(StructuredInput::Interleaved {
                files: self.segments.values().next().cloned().unwrap(),
                segment_order: interleaved.iter().map(|x| x.trim().to_string()).collect(),
            });
        } else {
            let mut segment_order: Vec<String> =
                self.segments.keys().map(|x| x.trim().to_string()).collect();
            segment_order.sort();
            if segment_order.is_empty() {
                bail!(
                    "No segments defined in input. At least one ('read1' perhaps?) must be defined."
                );
            }
            if segment_order.iter().any(|x| x == "all" || x == "All") {
                bail!(
                    "Segment name 'all' (or 'All') is reserved and cannot be used as a segment name."
                )
            }
            if segment_order
                .iter()
                .any(|x| x.eq_ignore_ascii_case("options"))
            {
                bail!(
                    "Segment name 'options' (any case) is reserved and cannot be used as a segment name."
                );
            }
            if segment_order.iter().any(|x| x.starts_with("_internal_")) {
                bail!(
                    "Segment names starting with '_internal_' are reserved and cannot be used as a segment name."
                )
            }

            self.structured = Some(StructuredInput::Segmented {
                segment_files: self.segments.clone(),
                segment_order,
            });
        }

        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => {
                let mut seen = HashSet::new();
                for key in segment_order {
                    if key.is_empty() || key.trim().is_empty() {
                        bail!("Segment name may not be empty (or just whitespace)");
                    }
                    if key.contains('/') || key.contains('\\') {
                        bail!(
                            "Segment name  may not contain path separators like / and \\. Was '{key}'",
                        );
                    }
                    if key.chars().any(|c| (c.is_ascii_control())) {
                        bail!("Segment name may not contain control characters. {key:?}");
                    }
                    /* if key.chars().any(|c| !(c.is_ascii())) {
                        bail!("Segment name may not contain non-ascii character");
                    } */

                    if !seen.insert(key) {
                        bail!("Segment name duplicated: '{key}'")
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, eserde::Deserialize)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, eserde::Deserialize)]
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
    }
    Ok(())
}

#[derive(eserde::Deserialize, Debug)]
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
}

impl Output {
    #[must_use]
    pub fn get_suffix(&self) -> String {
        self.format
            .get_suffix(self.compression, self.suffix.as_ref())
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Segment(pub String);

impl Default for Segment {
    fn default() -> Self {
        Segment(":::first_and_only_segment".to_string())
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SegmentOrAll(pub String);

impl Default for SegmentOrAll {
    fn default() -> Self {
        SegmentOrAll(":::first_and_only_segment".to_string())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub struct SegmentIndex(pub usize);

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize),
}

impl Segment {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(&mut self, input_def: &crate::config::Input) -> Result<SegmentIndex> {
        if self.0 == ":::first_and_only_segment" {
            if input_def.segment_count() == 1 {
                return Ok(SegmentIndex(0));
            } else {
                let segment_names = input_def.get_segment_order().join(", ");
                bail!(
                    "Segment not specified but multiple segments available: [{}]. \
                     Please specify which segment to use with 'segment = \"segment_name\"'",
                    segment_names
                );
            }
        }
        if self.0 == "all" || self.0 == "All" {
            bail!("'all' (or 'All') is not a valid segment in this position.");
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
        Ok(SegmentIndex(idx))
    }
}

impl SegmentOrAll {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<SegmentIndexOrAll> {
        if self.0 == ":::first_and_only_segment" {
            if input_def.segment_count() == 1 {
                return Ok(SegmentIndexOrAll::Indexed(0));
            } else {
                let segment_names = input_def.get_segment_order().join(", ");
                bail!(
                    "Segment not specified but multiple segments available: [{}]. \
                     Please specify which segment to use with 'segment = \"segment_name\"'",
                    segment_names
                );
            }
        }
        if self.0 == "all" || self.0 == "All" {
            return Ok(SegmentIndexOrAll::All);
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
        Ok(SegmentIndexOrAll::Indexed(idx))
    }
}

impl SegmentIndex {
    #[must_use]
    pub fn get_index(&self) -> usize {
        self.0
    }
}

impl TryInto<SegmentIndex> for SegmentIndexOrAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<SegmentIndex, Self::Error> {
        match self {
            SegmentIndexOrAll::Indexed(idx) => Ok(SegmentIndex(idx)),
            SegmentIndexOrAll::All => Err(()),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
    #[serde(default)]
    pub segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndex>,

    pub start: usize,
    #[validate(minimum = 1)]
    pub length: usize,
}

fn default_thread_count() -> usize {
    //num_cpus::get()
    2
}

fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

fn default_output_buffer_size() -> usize {
    1024 * 1024 // bytes, per fastq input file
}

fn default_block_size() -> usize {
    //todo: adjust depending on compression mode?
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

fn default_spot_check_read_pairing() -> bool {
    true
}

#[derive(eserde::Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct FailureOptions {
    #[serde(default)]
    pub fail_output_after_bytes: Option<usize>,
    #[serde(default)]
    pub fail_output_error: Option<FailOutputError>,
    #[serde(default)]
    pub fail_output_raw_os_code: Option<i32>,
}

impl FailureOptions {
    pub fn simulated_output_failure(&self) -> Result<Option<SimulatedWriteFailure>> {
        let Some(remaining_bytes) = self.fail_output_after_bytes else {
            return Ok(None);
        };

        let failure_error = self
            .fail_output_error
            .clone()
            .unwrap_or(FailOutputError::DiskFull);
        let error = match failure_error {
            FailOutputError::DiskFull => SimulatedWriteError::RawOs(28),
            FailOutputError::Other => SimulatedWriteError::Other,
            FailOutputError::RawOs => {
                let code = self
                    .fail_output_raw_os_code
                    .context(
                        "options.debug_failures.fail_output_raw_os_code required when fail_output_error = 'raw_os'",
                    )?;
                SimulatedWriteError::RawOs(code)
            }
        };

        Ok(Some(SimulatedWriteFailure {
            remaining_bytes: Some(remaining_bytes),
            error,
        }))
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FailOutputError {
    DiskFull,
    Other,
    RawOs,
}

#[derive(eserde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Options {
    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default = "default_output_buffer_size")]
    pub output_buffer_size: usize,
    #[serde(default)]
    pub accept_duplicate_files: bool,
    #[serde(default = "default_spot_check_read_pairing")]
    pub spot_check_read_pairing: bool,
    #[serde(default)]
    pub debug_failures: FailureOptions,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            thread_count: 10,
            block_size: default_block_size(),
            buffer_size: default_buffer_size(),
            output_buffer_size: default_output_buffer_size(),
            accept_duplicate_files: false,
            spot_check_read_pairing: default_spot_check_read_pairing(),
            debug_failures: FailureOptions::default(),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
pub struct Barcodes {
    #[serde(
        deserialize_with = "deser::btreemap_iupac_dna_string_from_string",
        flatten
    )]
    pub barcode_to_name: BTreeMap<BString, String>,
}

#[derive(eserde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub input: Input,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    pub output: Option<Output>,
    #[serde(default)]
    #[serde(alias = "step")]
    pub transform: Vec<Transformation>,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub barcodes: HashMap<String, Barcodes>,
}

impl Config {
    #[allow(clippy::too_many_lines)]
    pub fn check(&mut self) -> Result<()> {
        let mut errors = Vec::new();
        self.check_input_segment_definitions(&mut errors);
        if errors.is_empty() {
            //no point in checking them if segment definition is broken
            self.check_output(&mut errors);
            self.check_reports(&mut errors);
            self.check_barcodes(&mut errors);
            self.check_transformations(&mut errors);
            self.check_for_any_output(&mut errors);
            self.check_input_format(&mut errors);
        }

        // Return collected errors if any
        if !errors.is_empty() {
            if errors.len() == 1 {
                // For single errors, just return the error message directly
                bail!("{:?}", errors[0]);
            } else {
                // For multiple errors, format them cleanly
                let combined_error = errors
                    .into_iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<_>>()
                    .join("\n\n---------\n\n");
                bail!("Multiple errors occured:\n\n{}", combined_error);
            }
        }

        Ok(())
    }

    fn check_input_segment_definitions(&mut self, errors: &mut Vec<anyhow::Error>) {
        // Initialize segments and handle backward compatibility
        if let Err(e) = self.input.init() {
            errors.push(e);
            // Can't continue validation without proper segments
            if !errors.is_empty() {
                return;
            }
        }
    }

    fn check_input_format(&mut self, errors: &mut Vec<anyhow::Error>) {
        let mut seen = HashSet::new();
        if !self.options.accept_duplicate_files {
            // Check for duplicate files across all segments
            match self.input.structured.as_ref().unwrap() {
                StructuredInput::Interleaved { files, .. } => {
                    for f in files {
                        if !seen.insert(f.clone()) {
                            errors.push(anyhow!(
                                "[input]: Repeated filename: {} (in interleaved input). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                                f
                            ));
                        }
                    }
                }
                StructuredInput::Segmented {
                    segment_files,
                    segment_order,
                } => {
                    for segment_name in segment_order {
                        let files = segment_files.get(segment_name).unwrap();
                        if files.is_empty() {
                            errors.push(anyhow!(
                                "[input]: Segment '{}' has no files specified.",
                                segment_name
                            ));
                        }
                        for f in files {
                            if !seen.insert(f.clone()) {
                                errors.push(anyhow!(
                                    "[input]: Repeated filename: {} (in segment '{}'). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                                    f, segment_name
                                ));
                            }
                        }
                    }
                }
            }
        }

        let mut saw_fasta = false;
        let mut saw_bam = false;
        match self.input.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { files, .. } => {
                let mut interleaved_format: Option<DetectedInputFormat> = None;
                for filename in files {
                    match io::detect_input_format(Path::new(filename)) {
                        Ok(format) => {
                            if let Some(existing) = interleaved_format {
                                if existing != format {
                                    errors.push(anyhow!(
                                        "[input]: Interleaved inputs must all have the same format. Found both {existing:?} and {format:?} when reading {filename}."
                                    ));
                                }
                            } else {
                                interleaved_format = Some(format);
                            }
                            match format {
                                DetectedInputFormat::Fastq => {}
                                DetectedInputFormat::Fasta => saw_fasta = true,
                                DetectedInputFormat::Bam => saw_bam = true,
                            }
                        }
                        Err(_) => {
                            //ignore for now. We'll complain again later,
                            //but here we're only checking the consistency within the configuration
                        } /* errors.push(
                          e.context(format!(
                              "[input]: Failed to detect input format for interleaved file '{filename}'."
                          )),) */
                          ,
                    }
                }
            }
            StructuredInput::Segmented {
                segment_order,
                segment_files,
            } => {
                for segment_name in segment_order {
                    let mut segment_format: Option<DetectedInputFormat> = None;
                    if let Some(files) = segment_files.get(segment_name) {
                        for filename in files {
                            match io::detect_input_format(Path::new(filename)) {
                                Ok(format) => {
                                    if let Some(existing) = segment_format {
                                        if existing != format {
                                            errors.push(anyhow!(
                                                "[input]: Segment '{segment_name}' mixes input formats {existing:?} and {format:?}. Use separate segments per format."
                                            ));
                                        }
                                    } else {
                                        segment_format = Some(format);
                                    }
                                    match format {
                                        DetectedInputFormat::Fastq => {}
                                        DetectedInputFormat::Fasta => saw_fasta = true,
                                        DetectedInputFormat::Bam => saw_bam = true,
                                    }
                                }
                                Err(_) => {
                                    //ignore for now. We'll complain again later,
                                    //but here we're only checking the consistency within the configuration
                                } /* errors.push(
                                      e.context(format!(
                                          "[input]: Failed to detect input format for file '{filename}' in segment '{segment_name}'."
                                      )),
                                  ), */
                            }
                        }
                    }
                }
            }
        }

        if saw_fasta {
            if self.input.options.fasta_fake_quality.is_none() {
                errors.push(anyhow!(
                    "[input.options]: 'fasta_fake_quality' must be set when reading FASTA inputs."
                ));
            }
        }

        if saw_bam {
            let include_mapped = self.input.options.bam_include_mapped;
            let include_unmapped = self.input.options.bam_include_unmapped;
            if include_mapped.is_none() {
                errors.push(anyhow!(
                    "[input.options]: 'bam_include_mapped' must be set (true or false) when reading BAM inputs."
                ));
            }
            if include_unmapped.is_none() {
                errors.push(anyhow!(
                    "[input.options]: 'bam_include_unmapped' must be set (true or false) when reading BAM inputs."
                ));
            } else if include_mapped == Some(false) && include_unmapped == Some(false) {
                errors.push(anyhow!(
                    "[input.options]: At least one of 'bam_include_mapped' or 'bam_include_unmapped' must be true when reading BAM inputs."
                ));
            }
        }

        if self.options.block_size % 2 == 1 && self.input.interleaved.is_some() {
            errors.push(anyhow!(
                "[options]: Block size must be even for interleaved input."
            ));
        }
    }

    fn check_transform_segments(&mut self, errors: &mut Vec<anyhow::Error>) {
        // check each transformation, validate labels
        for (step_no, t) in self.transform.iter_mut().enumerate() {
            // dbg!(&t);
            if let Err(e) = t.validate_segments(&self.input) {
                errors.push(e.context(format!("[Step {step_no} ({t})]")));
            }
        }
    }

    fn check_transformations(&mut self, errors: &mut Vec<anyhow::Error>) {
        #[derive(Debug)]
        struct TagMetadata {
            used: bool,
            declared_at_step: usize,
            declared_by: String,
            tag_type: TagValueType,
        }

        self.check_transform_segments(errors);
        if !errors.is_empty() {
            return; // Can't continue validation if segments are invalid
        }
        let mut tags_available: HashMap<String, TagMetadata> = HashMap::new();

        // Resolve config references after basic validation but before other checks
        let barcodes_data = self.barcodes.clone();
        for (step_no, t) in self.transform.iter_mut().enumerate() {
            if let Err(e) = t.resolve_config_references(&barcodes_data) {
                errors.push(e.context(format!("[Step {step_no} ({t})]:")));
            }
        }

        for (step_no, t) in self.transform.iter().enumerate() {
            if let Err(e) =
                t.validate_others(&self.input, self.output.as_ref(), &self.transform, step_no)
            {
                errors.push(e.context(format!("[Step {step_no} ({t})]:")));
                continue; // Skip further processing of this transform if validation failed
            }

            if let Some((tag_name, tag_type)) = t.declares_tag_type() {
                if tag_name.is_empty() {
                    errors.push(anyhow!("[Step {step_no} ({t})]: Label cannot be empty"));
                    continue;
                }
                if tag_name == "ReadName" {
                    // because that's what we store in the output tables as
                    // column 0
                    errors.push(anyhow!("[Step {step_no} ({t})]: Reserved tag name 'ReadName' cannot be used as a tag label"));
                    continue;
                }
                if tags_available.contains_key(&tag_name) {
                    errors.push(anyhow!(
                        "[Step {step_no} ([{t})]: Duplicate label: {tag_name}. Each tag must be unique",
                    ));
                    continue;
                }
                tags_available.insert(
                    tag_name.clone(),
                    TagMetadata {
                        used: false,
                        declared_at_step: step_no,
                        declared_by: t.to_string(),
                        tag_type: tag_type,
                    },
                );
            }

            if let Some(tags_to_remove) = t.removes_tags() {
                for tag_name in tags_to_remove {
                    //no need to check if empty, empty will never be present
                    if let Some(metadata) = tags_available.get_mut(&tag_name) {
                        metadata.used = true;
                    } else {
                        errors.push(anyhow!(
                        "[Step {step_no} ({t})]: Can't remove tag {tag_name}, not present. Available at this point: {tags_available:?}. Transform: {t}"
                    ));
                        continue;
                    }
                    tags_available.remove(&tag_name);
                }
            }

            if t.uses_all_tags() {
                for metadata in tags_available.values_mut() {
                    metadata.used = true;
                }
            }
            if let Some(tag_names_and_types) = t.uses_tags() {
                for (tag_name, tag_type) in tag_names_and_types {
                    //no need to check if empty, empty will never be present
                    let entry = tags_available.get_mut(&tag_name);
                    match entry {
                        Some(metadata) => {
                            metadata.used = true;
                            if !tag_type.compatible(metadata.tag_type) {
                                errors.push(anyhow!  (
                            "[Step {step_no} ({t})]: Tag '{label}' does not provide the required tag type '{supposed_tag_type}'. It provides '{actual_tag_type}'.", supposed_tag_type=tag_type, label=tag_name, actual_tag_type=metadata.tag_type ));
                            }
                        }
                        None => {
                            errors.push(anyhow!(
                                "[Step {step_no} ({t})]: No step generating label '{tag_name}' (or removed previously). Available at this point: {{{}}}.", tags_available.keys().cloned().collect::<Vec<_>>().join(", ")
                            ));
                        }
                    }
                }
            }
        }
        for (tag_name, metadata) in tags_available.iter().filter(|(_, meta)| !meta.used) {
            errors.push(anyhow!(
                "[Step {declared_at_step} ({declared_by})]: Extract label '{tag_name}' (type {tag_type}) is never used downstream.",
                declared_at_step = metadata.declared_at_step,
                tag_name = tag_name,
                declared_by = metadata.declared_by,
                tag_type = metadata.tag_type,
            ));
        }
    }

    fn check_output(&mut self, errors: &mut Vec<anyhow::Error>) {
        //apply output if set
        if let Some(output) = &mut self.output {
            if output.format == FileFormat::Bam {
                if output.output_hash_uncompressed {
                    errors.push(anyhow!(
                        "[output]: Uncompressed hashing is not supported when format = 'bam'. Set output_hash_uncompressed = false.",
                    ));
                }
                if output.stdout {
                    errors.push(anyhow!(
                        "[output]: format = 'bam' cannot be used together with stdout output.",
                    ));
                }
                if output.compression != CompressionFormat::Uncompressed {
                    errors.push(anyhow!(
                        "[output]: Compression cannot be specified when format = 'bam'. Remove the compression setting.",
                    ));
                }
            }
            if output.stdout {
                if output.output.is_some() {
                    errors.push(anyhow!(
                        "[output]: Cannot specify both 'stdout' and 'output' options together. You need to use 'interleave' to control which segments to output to stdout" 
                    ));
                }
                /* if output.format != FileFormat::Bam {
                output.format = FileFormat::Fastq;
                output.compression = CompressionFormat::Uncompressed; */
                //}
                if output.interleave.is_none() {
                    output.interleave = Some(self.input.get_segment_order().clone());
                }
            } else if output.output.is_none() {
                if output.interleave.is_some() {
                    output.output = Some(Vec::new()); // no extra output by default
                } else {
                    //default to output all targets
                    output.output = Some(self.input.get_segment_order().clone());
                }
            }

            if let Some(interleave_order) = output.interleave.as_ref() {
                let valid_segments: HashSet<&String> =
                    self.input.get_segment_order().iter().collect();
                let mut seen_segments = HashSet::new();
                for segment in interleave_order {
                    if !valid_segments.contains(segment) {
                        errors.push(anyhow!(
                            "[output]: Interleave segment '{}' not found in input segments: {:?}",
                            segment,
                            valid_segments
                        ));
                    }
                    if !seen_segments.insert(segment) {
                        errors.push(anyhow!(
                            "[output]: Interleave segment '{}' is duplicated in interleave order: {:?}",
                            segment,
                            interleave_order
                        ));
                    }
                }
                if interleave_order.len() < 2 && !output.stdout {
                    errors.push(anyhow!(
                        "[output]: Interleave order must contain at least two segments to interleave. Got: {:?}",
                        interleave_order
                    ));
                }
                //make sure there's no overlap between interleave and output
                if let Some(output_segments) = output.output.as_ref() {
                    for segment in output_segments {
                        if interleave_order.contains(segment) {
                            errors.push(anyhow!(
                                "[output]: Segment '{}' cannot be both in 'interleave' and 'output' lists. Interleave: {:?}, Output: {:?}",
                                segment,
                                interleave_order,
                                output_segments
                            ));
                        }
                    }
                }
            }

            // Validate compression level for output
            if let Err(e) =
                validate_compression_level_u8(output.compression, output.compression_level)
            {
                errors.push(anyhow!("[output]: {}", e));
            }
        }
    }
    fn check_reports(&self, errors: &mut Vec<anyhow::Error>) {
        let report_html = self.output.as_ref().is_some_and(|o| o.report_html);
        let report_json = self.output.as_ref().is_some_and(|o| o.report_json);
        let has_report_transforms = self.transform.iter().any(|t| {
            matches!(t, Transformation::Report { .. })
                | matches!(t, Transformation::_InternalReadCount { .. })
        });

        if has_report_transforms && !(report_html || report_json) {
            errors.push(anyhow!(
                "[output]: Report step configured, but neither output.report_json nor output.report_html is true. Enable at least one to write report files.",
            ));
        }

        if (report_html || report_json) && !has_report_transforms {
            errors.push(anyhow!("[output]: Report (html|json) requested, but no report step in configuration. Either disable the reporting, or add a
\"\"\"
[step]
    type = \"report\"
    count = true
    ...
\"\"\" section"));
        }
    }

    fn check_for_any_output(&self, errors: &mut Vec<anyhow::Error>) {
        let has_fastq_output = self.output.as_ref().is_some_and(|o| {
            o.stdout
                || o.output.as_ref().is_none_or(|o| !o.is_empty())
                || o.interleave.as_ref().is_some_and(|i| !i.is_empty())
        });
        let has_report_output = self
            .output
            .as_ref()
            .is_some_and(|o| o.report_html || o.report_json);
        let has_tag_output = self.transform.iter().any(|t| {
            matches!(
                t,
                Transformation::StoreTagInFastQ { .. }
                    | Transformation::StoreTagsInTable { .. }
                    | Transformation::Inspect { .. }
            )
        });

        if !has_fastq_output && !has_report_output && !has_tag_output {
            errors.push(anyhow!(
                "[output]: No output files and no reports requested. Nothing to do."
            ));
        }
    }

    fn check_barcodes(&self, errors: &mut Vec<anyhow::Error>) {
        // Check that barcode names are unique across all barcodes sections
        for (section_name, barcodes) in &self.barcodes {
            if barcodes.barcode_to_name.values().any(|x| x == "no-barcode") {
                errors.push(anyhow!(
                    "[barcodes.{section_name}]: Barcode output infix must not be 'no-barcode'"
                ));
            }

            if barcodes.barcode_to_name.is_empty() {
                errors.push(anyhow!(
                    "[barcodes.{section_name}]: Barcode section must contain at least one barcode mapping",
                ));
            }

            // assert that barcodes have all the same length
            let lengths: HashSet<usize> =
                barcodes.barcode_to_name.keys().map(|b| b.len()).collect();
            if lengths.len() > 1 {
                dbg!(&barcodes);
                errors.push(anyhow!(
                    "[barcodes.{section_name}]: All barcodes in one section must have the same length. Observed: {lengths:?}.",
                ));
            }

            // Check for overlapping IUPAC barcodes
            if let Err(e) = validate_barcode_disjointness(&barcodes.barcode_to_name) {
                errors.push(anyhow!("[barcodes.{}]: {}", section_name, e));
            }
        }
    }
}

/// Validate that IUPAC barcodes are disjoint (don't overlap in their accepted sequences)
#[allow(clippy::collapsible_if)]
fn validate_barcode_disjointness(barcodes: &BTreeMap<BString, String>) -> Result<()> {
    let barcode_patterns: Vec<_> = barcodes.iter().collect();

    for i in 0..barcode_patterns.len() {
        for j in (i + 1)..barcode_patterns.len() {
            if crate::dna::iupac_overlapping(barcode_patterns[i].0, barcode_patterns[j].0) {
                if barcode_patterns[i].1 != barcode_patterns[j].1 {
                    bail!(
                        "Barcodes '{}' and '{}' have overlapping accepted sequences but lead to different outputs. Must be disjoint.",
                        String::from_utf8_lossy(barcode_patterns[i].0),
                        String::from_utf8_lossy(barcode_patterns[j].0)
                    );
                }
            }
        }
    }
    Ok(())
}

#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub enum PhredEncoding {
    #[serde(alias = "sanger")]
    #[serde(alias = "illumina_1_8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "Illumina_1_8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "illumina1.8")] //ilummina 1.8+ is sanger.
    #[serde(alias = "Illumina1.8")] //ilummina 1.8+ is sanger.
    Sanger, //33..=126, offset 33
    #[serde(alias = "solexa")]
    Solexa, //59..=126, offset 64
    #[serde(alias = "illumina_1_3")]
    #[serde(alias = "Illumina_1_3")]
    #[serde(alias = "illumina1.3")]
    #[serde(alias = "Illumina1.3")]
    Illumina13, //64..=126, offset 64
}

impl PhredEncoding {
    #[must_use]
    pub fn limits(&self) -> (u8, u8) {
        match self {
            PhredEncoding::Sanger => (33, 126),
            PhredEncoding::Solexa => (59, 126),
            PhredEncoding::Illumina13 => (64, 126),
        }
    }
}
