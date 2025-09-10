#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
use crate::transformations::{Step, Transformation};
use anyhow::{bail, Result};
use serde_valid::Validate;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fmt::Display,
};

pub mod deser;

use deser::deserialize_map_of_string_or_seq_string;

#[derive(eserde::Deserialize, Debug, Clone, serde::Serialize)]
pub struct Input {
    #[serde(default)]
    interleaved: bool,
    #[serde(flatten, deserialize_with = "deserialize_map_of_string_or_seq_string")]
    segments: HashMap<String, Vec<String>>,

    // Computed field for consistent ordering - not serialized
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub segment_order: Vec<String>,
}

impl Input {
    /// Initialize the segment_order and handle backward compatibility

    /// Get all segment names in order
    pub fn get_segment_order(&self) -> &[String] {
        &self.segment_order
    }

    /// Get files for a specific segment
    pub fn get_segment_files(&self, segment_name: &str) -> Option<&Vec<String>> {
        self.segments.get(segment_name)
    }

    fn init(&mut self) -> Result<()> {
        // Determine segment order: read1, read2, index1, index2 first if present, then others alphabetically
        let mut other_segments: Vec<String> = self.segments.keys().cloned().collect();
        other_segments.sort();
        self.segment_order = other_segments;

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

#[derive(eserde::Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Target {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Target::Read1 => write!(f, "Read1"),
            Target::Read2 => write!(f, "Read2"),
            Target::Index1 => write!(f, "Index1"),
            Target::Index2 => write!(f, "Index2"),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Copy, Clone)]
pub enum TargetPlusAll {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
    #[serde(alias = "all")]
    All,
}

impl Display for TargetPlusAll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetPlusAll::Read1 => write!(f, "Read1"),
            TargetPlusAll::Read2 => write!(f, "Read2"),
            TargetPlusAll::Index1 => write!(f, "Index1"),
            TargetPlusAll::Index2 => write!(f, "Index2"),
            TargetPlusAll::All => write!(f, "All"),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
    pub source: Target,
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
            for segment_name in self.input.get_segment_order() {
                let files = self.input.get_segment_files(&segment_name).unwrap();
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

        if !self.input.get_segment_files("read1").is_some() {
            errors.push(anyhow::anyhow!(
                "No 'read1' segment found in input. At least 'read1' must be specified. For now",
            ));
        }
        /* if self.input.interleaved && has_read2 {
                   errors.push(anyhow::anyhow!(
                       "[input]: If interleaved is set, read2 segment must not be present"
                   ));
               }
        */
        /* if let Some(output) = &self.output {
            if output.interleave && !has_read2 {
                errors.push(anyhow::anyhow!(
                    "[input]: Interleaving requires read2 segment to be specified."
                ));
            }
        } */

        if self.options.block_size % 2 == 1 && self.input.interleaved{
            errors.push(anyhow::anyhow!(
                "[options]: Block size must be even for interleaved input."
            ));
        }
    }

    fn check_transformations(&self, errors: &mut Vec<anyhow::Error>) {
        let mut tags_available: HashMap<String, bool> = HashMap::new();
        // check each transformation, validate labels
        for (step_no, t) in self.transform.iter().enumerate() {
            if let Err(e) = t.validate(&self.input, self.output.as_ref(), &self.transform, step_no)
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
                if self.input.get_segment_order().len() > 1 {
                    if output.interleave.is_none() {
                        output.interleave =
                            Some(self.input.get_segment_order().iter().cloned().collect())
                    }
                } else {
                    output.interleave = None;
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
                if interleave_order.len() < 2 {
                    errors.push(anyhow::anyhow!(
                        "[output]: Interleave order must contain at least two segments to interleave. Got: {:?}",
                        interleave_order
                    ));
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
