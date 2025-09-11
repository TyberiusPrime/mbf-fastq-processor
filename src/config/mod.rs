#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
use crate::transformations::{Step, Transformation};
use anyhow::{bail, Context, Result};
use serde_valid::Validate;
use std::collections::{BTreeMap, HashMap, HashSet};

pub mod deser;

use deser::deserialize_map_of_string_or_seq_string;

#[derive(eserde::Deserialize, Debug, Clone, serde::Serialize)]
pub struct Input {
    #[serde(default)]
    interleaved: Option<Vec<String>>,
    #[serde(flatten, deserialize_with = "deserialize_map_of_string_or_seq_string")]
    segments: HashMap<String, Vec<String>>,

    // Computed field for consistent ordering - not serialized
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub structured: Option<StructuredInput>,
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
    pub fn segment_count(&self) -> usize {
        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order.len(),
        }
    }

    pub fn get_segment_order(&self) -> &Vec<String> {
        match self.structured.as_ref().unwrap() {
            StructuredInput::Interleaved { segment_order, .. }
            | StructuredInput::Segmented { segment_order, .. } => segment_order,
        }
    }

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
                .map(|(k, v)| format!("\t'{}': \t{}", k, v))
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
                segment_order: interleaved.iter().cloned().collect(),
            })
        } else {
            let mut segment_order: Vec<String> = self.segments.keys().cloned().collect();
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
            self.structured = Some(StructuredInput::Segmented {
                segment_files: self.segments.clone(),
                segment_order,
            })
        }

        // Determine segment order: read1, read2, index1, index2 first if present, then others alphabetically
        Ok(())
    }
}

#[derive(eserde::Deserialize, Debug, Copy, Clone, Default)]
pub enum FileFormat {
    #[serde(alias = "raw")]
    #[serde(alias = "uncompressed")]
    #[serde(alias = "Uncompressed")]
    #[default]
    Raw,
    #[serde(alias = "gzip")]
    #[serde(alias = "gz")]
    #[serde(alias = "Gz")]
    Gzip,
    #[serde(alias = "zstd")]
    #[serde(alias = "zst")]
    #[serde(alias = "Zst")]
    Zstd,
    #[serde(alias = "none")] // we need this so you can disable the output, but set a prefix for
    // the Reports
    None,
}

impl FileFormat {
    #[must_use]
    pub fn get_suffix(&self, custom_suffix: Option<&String>) -> String {
        custom_suffix
            .map_or_else(
                || match self {
                    FileFormat::Raw => "fq",
                    FileFormat::Gzip => "fq.gz",
                    FileFormat::Zstd => "fq.zst",
                    FileFormat::None => "",
                },
                |s| s.as_str(),
            )
            .to_string()
    }
}

/// Validates that the compression level is within the expected range for the given file format
pub fn validate_compression_level_u8(
    format: FileFormat,
    compression_level: Option<u8>,
) -> Result<(), String> {
    if let Some(level) = compression_level {
        match format {
            FileFormat::Raw | FileFormat::None => {
                if level != 0 {
                    return Err(format!(
                        "Compression level {level} specified for format {format:?}, but raw/none formats don't use compression",
                    ));
                }
            }
            FileFormat::Gzip => {
                if level > 9 {
                    return Err(format!(
                        "Compression level {level} is invalid for gzip format. Valid range is 0-9.",
                    ));
                }
            }
            FileFormat::Zstd => {
                if level > 22 || level == 0 {
                    return Err(format!(
                        "Compression level {level} is invalid for zstd format. Valid range is 1-22.",
                    ));
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
        self.format.get_suffix(self.suffix.as_ref())
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Segment(pub String);

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct SegmentOrAll(pub String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SegmentIndex(pub usize, pub String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SegmentIndexOrAll {
    All,
    Indexed(usize, String),
}

impl Segment {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(&mut self, input_def: &crate::config::Input) -> Result<SegmentIndex> {
        if self.0 == "all" || self.0 == "All" {
            bail!("'all' (or 'All') is not a valid segment in this position.");
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
        Ok(SegmentIndex(idx, self.0.clone()))
    }
}

impl SegmentOrAll {
    /// validate and turn into an indexed segment
    pub(crate) fn validate(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<SegmentIndexOrAll> {
        if self.0 == "all" || self.0 == "All" {
            return Ok(SegmentIndexOrAll::All);
        }
        let name = &self.0;
        let idx = input_def
            .index(name)
            .with_context(|| format!("Unknown segment: {name}"))?;
        Ok(SegmentIndexOrAll::Indexed(idx, self.0.clone()))
    }
}

impl SegmentIndex {
    pub fn get_index(&self) -> usize {
        self.0
    }

    pub fn get_name(&self) -> &str {
        &self.1
    }
}

impl TryInto<SegmentIndex> for &SegmentIndexOrAll {
    type Error = ();

    fn try_into(self) -> std::prelude::v1::Result<SegmentIndex, Self::Error> {
        match self {
            SegmentIndexOrAll::Indexed(idx, name) => Ok(SegmentIndex(*idx, name.clone())),
            SegmentIndexOrAll::All => Err(()),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
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
}

impl Default for Options {
    fn default() -> Self {
        Options {
            thread_count: 10,
            block_size: default_block_size(),
            buffer_size: default_buffer_size(),
            output_buffer_size: default_output_buffer_size(),
            accept_duplicate_files: false,
        }
    }
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
}

impl Config {
    #[allow(clippy::too_many_lines)]
    pub fn check(&mut self) -> Result<()> {
        let mut errors = Vec::new();
        self.check_input(&mut errors);
        if errors.is_empty() {
            //no point in checking them if input is broken
            self.check_output(&mut errors);
            self.check_reports(&mut errors);
            self.check_transformations(&mut errors);
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

    fn check_input(&mut self, errors: &mut Vec<anyhow::Error>) {
        // Initialize segments and handle backward compatibility
        if let Err(e) = self.input.init() {
            errors.push(e);
            // Can't continue validation without proper segments
            if !errors.is_empty() {
                return;
            }
        }
        let mut seen = HashSet::new();
        if !self.options.accept_duplicate_files {
            // Check for duplicate files across all segments
            match self.input.structured.as_ref().unwrap() {
                StructuredInput::Interleaved { files, .. } => {
                    for f in files {
                        if !seen.insert(f.clone()) {
                            errors.push(anyhow::anyhow!(
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
                            errors.push(anyhow::anyhow!(
                                "[input]: Segment '{}' has no files specified.",
                                segment_name
                            ));
                        }
                        for f in files {
                            if !seen.insert(f.clone()) {
                                errors.push(anyhow::anyhow!(
                                    "[input]: Repeated filename: {} (in segment '{}'). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                                    f, segment_name
                                ));
                            }
                        }
                    }
                }
            }
        }

        if self.options.block_size % 2 == 1 && self.input.interleaved.is_some() {
            errors.push(anyhow::anyhow!(
                "[options]: Block size must be even for interleaved input."
            ));
        }
    }

    fn check_transformations(&mut self, errors: &mut Vec<anyhow::Error>) {
        let mut tags_available: HashMap<String, bool> = HashMap::new();
        // check each transformation, validate labels
        for (step_no, t) in self.transform.iter_mut().enumerate() {
            // dbg!(&t);
            if let Err(e) = t.validate_segments(&self.input) {
                errors.push(e.context(format!("[Step {step_no}]: {t}")));
            }
        }
        for (step_no, t) in self.transform.iter().enumerate() {
            if let Err(e) =
                t.validate_others(&self.input, self.output.as_ref(), &self.transform, step_no)
            {
                errors.push(e.context(format!("[Step {step_no}]: {t}")));
                continue; // Skip further processing of this transform if validation failed
            }

            if let Some(tag_name) = t.sets_tag() {
                if tag_name.is_empty() {
                    errors.push(anyhow::anyhow!(
                        "[Step {step_no}]: Extract* label cannot be empty. Transform: {t}"
                    ));
                    continue;
                }
                if tag_name == "ReadName" {
                    errors.push(anyhow::anyhow!("[Step {step_no}]: Reserved tag name 'ReadName' cannot be used as a tag label. Transform: {t}"));
                    continue;
                }
                if tags_available
                    .insert(tag_name, t.tag_provides_location())
                    .is_some()
                {
                    errors.push(anyhow::anyhow!(
                        "[Step {step_no}]: Duplicate extract label: {tag_name}. Each tag must be unique.. Transform: {t}",
                        tag_name = t.sets_tag().unwrap()
                    ));
                    continue;
                }
            }
            if let Some(tag_name) = t.removes_tag() {
                //no need to check if empty, empty will never be present
                if !tags_available.contains_key(&tag_name) {
                    errors.push(anyhow::anyhow!(
                        "[Step {step_no}]: Can't remove tag {tag_name}, not present. Available at this point: {tags_available:?}. Transform: {t}"
                    ));
                    continue;
                }
                tags_available.remove(&tag_name);
            }
            if let Some(tag_names) = t.uses_tags() {
                for tag_name in tag_names {
                    //no need to check if empty, empty will never be present
                    let entry = tags_available.get(&tag_name);
                    match entry {
                        Some(provides_location) => {
                            if !provides_location && t.tag_requires_location() {
                                errors.push(anyhow::anyhow!(
                                    "[Step {step_no}]: Tag '{tag_name}' does not provide location data required by '{step_name}'",
                                    tag_name = tag_name,
                                    step_name = t.to_string()
                                ));
                            }
                        }
                        None => {
                            errors.push(anyhow::anyhow!(
                                "[Step {step_no}]: No Extract* generating label '{tag_name}' (or removed previously). Available at this point: {tags_available:?}. Transform: {t}"
                            ));
                        }
                    }
                }
            }
        }
    }

    fn check_output(&mut self, errors: &mut Vec<anyhow::Error>) {
        //apply output if set
        if let Some(output) = &mut self.output {
            if output.stdout {
                if output.output.is_some() {
                    errors.push(anyhow::anyhow!(
                        "[output]: Cannot specify both 'stdout' and 'output' options together."
                    ));
                }
                output.format = FileFormat::Raw;
                if output.interleave.is_none() {
                    output.interleave =
                        Some(self.input.get_segment_order().iter().cloned().collect())
                }
            } else {
                if output.output.is_none() {
                    if output.interleave.is_some() {
                        output.output = Some(Vec::new()); // no extra output by default
                    } else {
                        //default to output all targets
                        output.output =
                            Some(self.input.get_segment_order().iter().cloned().collect());
                    }
                }
            }

            if let Some(interleave_order) = output.interleave.as_ref() {
                let valid_segments: HashSet<&String> =
                    self.input.get_segment_order().iter().collect();
                let mut seen_segments = HashSet::new();
                for segment in interleave_order {
                    if !valid_segments.contains(segment) {
                        errors.push(anyhow::anyhow!(
                            "[output]: Interleave segment '{}' not found in input segments: {:?}",
                            segment,
                            valid_segments
                        ));
                    }
                    if !seen_segments.insert(segment) {
                        errors.push(anyhow::anyhow!(
                            "[output]: Interleave segment '{}' is duplicated in interleave order: {:?}",
                            segment,
                            interleave_order
                        ));
                    }
                }
                if interleave_order.len() < 2 && !output.stdout {
                    errors.push(anyhow::anyhow!(
                        "[output]: Interleave order must contain at least two segments to interleave. Got: {:?}",
                        interleave_order
                    ));
                }
                //make sure there's no overlap between interleave and output
                if let Some(output_segments) = output.output.as_ref() {
                    for segment in output_segments {
                        if interleave_order.contains(segment) {
                            errors.push(anyhow::anyhow!(
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
            if let Err(e) = validate_compression_level_u8(output.format, output.compression_level) {
                errors.push(anyhow::anyhow!("[output]: {}", e));
            }
        }
    }
    fn check_reports(&self, errors: &mut Vec<anyhow::Error>) {
        let report_html = self.output.as_ref().is_some_and(|o| o.report_html);
        let report_json = self.output.as_ref().is_some_and(|o| o.report_json);

        if report_html || report_json {
            let has_report_transforms = self.transform.iter().any(|t| {
                matches!(t, Transformation::Report { .. })
                    | matches!(t, Transformation::_InternalReadCount { .. })
            });
            if !has_report_transforms {
                errors.push(anyhow::anyhow!("[output]: Report (html|json) requested, but no report step in configuration. Either disable the reporting, or add a
\"\"\"
[step]
    type = \"report\"
    count = true
    ...
\"\"\" section"));
            }
        }
    }
}
