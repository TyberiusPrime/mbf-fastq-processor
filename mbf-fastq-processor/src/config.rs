#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
//
use crate::io::{self, DetectedInputFormat};
use crate::transformations::{Step, TagValueType, Transformation};
use anyhow::{Result, anyhow, bail};
use bstr::BString;
use schemars::JsonSchema;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod deser;
mod input;
pub mod options;
mod output;
mod segments;

pub use crate::io::fileformats::PhredEncoding;
pub use input::{
    CompressionFormat, FileFormat, Input, InputOptions, STDIN_MAGIC_PATH, StructuredInput,
    validate_compression_level_u8,
};
pub use options::Options;
pub use output::Output;
pub use segments::{
    Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll, SegmentOrNameIndex,
    SegmentSequenceOrName,
};

#[derive(Debug)]
pub struct TagMetadata {
    pub used: bool,
    pub declared_at_step: usize,
    pub declared_by: String,
    pub tag_type: TagValueType,
}

/// Validates that a tag name conforms to the pattern [a-zA-Z_][a-zA-Z0-9_]*
/// (starts with a letter or underscore, followed by zero or more alphanumeric characters or underscores)
pub fn validate_tag_name(tag_name: &str) -> Result<()> {
    if tag_name.is_empty() {
        bail!(
            "Tag label cannot be empty. Please provide a non-empty tag name that starts with a letter or underscore."
        );
    }

    let mut chars = tag_name.chars();
    let first_char = chars
        .next()
        .expect("tag_name is not empty so must have at least one char");

    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        bail!("Tag label must start with a letter or underscore (a-zA-Z_), got '{first_char}'",);
    }

    for (i, ch) in chars.enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Tag label must contain only letters, numbers, and underscores (a-zA-Z0-9_), found '{ch}' at position {}",
                i + 1
            );
        }
    }
    if tag_name == "ReadName" {
        // because that's what we store in the output tables as
        // column 0
        bail!(
            "Reserved tag label 'ReadName' cannot be used as a tag label. This name is reserved for the read name column in output tables. Please choose a different tag name."
        );
    }
    if tag_name.starts_with("len_") {
        bail!(
            "Tag label '{tag_name}' cannot start with reserved prefix 'len_'. This prefix is reserved for length-related internal tags. Please choose a different tag name that doesn't start with 'len_'."
        );
    }
    Ok(())
}

/// Validates that a segment label conforms to the pattern [a-zA-Z0-9_]+
/// (one or more alphanumeric characters or underscores)
pub fn validate_segment_label(label: &str) -> Result<()> {
    if label.is_empty() {
        bail!(
            "Segment name may not be empty or just whitespace. Please provide a segment name containing only letters, numbers, and underscores."
        );
    }

    for (i, ch) in label.chars().enumerate() {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Segment label must contain only letters, numbers, and underscores (^[a-zA-Z0-9_]+$), found '{ch}' at position {i}",
            );
        }
    }

    Ok(())
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema, serde_valid::Validate)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    /// Enable benchmark mode
    #[serde(default)]
    pub enable: bool,

    #[serde(default)]
    pub quiet: bool,
    /// Number of molecules to process in benchmark mode
    #[serde(default)]
    #[validate(minimum = 1)]
    pub molecule_count: usize,
}

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The input configuration
    pub input: Input,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    pub output: Option<Output>,
    #[serde(default)]
    #[serde(alias = "step")]
    pub transform: Vec<Transformation>,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub barcodes: BTreeMap<String, Barcodes>,
    #[serde(default)]
    pub benchmark: Option<Benchmark>,

    #[serde(skip)]
    #[serde(default)]
    pub report_labels: Vec<String>,
}

fn expand_reports(
    res: &mut Vec<Transformation>,
    res_report_labels: &mut Vec<String>,
    report_no: &mut usize,
    config: crate::transformations::reports::Report,
) {
    use crate::transformations::prelude::DemultiplexedData;
    use crate::transformations::reports;
    res.push(Transformation::Report(config.clone())); // for validation. We remove it again  in
    // Transformation::Expand
    res_report_labels.push(config.name);
    if config.count {
        res.push(Transformation::_ReportCount(Box::new(
            reports::_ReportCount::new(*report_no),
        )));
    }
    if config.length_distribution {
        res.push(Transformation::_ReportLengthDistribution(Box::new(
            reports::_ReportLengthDistribution::new(*report_no),
        )));
    }
    if config.duplicate_count_per_read {
        res.push(Transformation::_ReportDuplicateCount(Box::new(
            reports::_ReportDuplicateCount {
                report_no: *report_no,
                data_per_segment: Arc::new(Mutex::new(DemultiplexedData::default())),
                debug_reproducibility: config.debug_reproducibility,
                initial_filter_capacity: Arc::new(Mutex::new(None)),
                actual_filter_capacity: None,
            },
        )));
    }
    if config.duplicate_count_per_fragment {
        res.push(Transformation::_ReportDuplicateFragmentCount(Box::new(
            reports::_ReportDuplicateFragmentCount {
                report_no: *report_no,
                data: Arc::new(Mutex::new(DemultiplexedData::default())),
                debug_reproducibility: config.debug_reproducibility,
                initial_filter_capacity: Arc::new(Mutex::new(None)),
                actual_filter_capacity: None,
            },
        )));
    }
    if config.base_statistics {
        use crate::transformations::reports;
        res.push(Transformation::_ReportBaseStatisticsPart1(Box::new(
            reports::_ReportBaseStatisticsPart1::new(*report_no),
        )));
        res.push(Transformation::_ReportBaseStatisticsPart2(Box::new(
            reports::_ReportBaseStatisticsPart2::new(*report_no),
        )));
    }

    if let Some(count_oligos) = config.count_oligos.as_ref() {
        res.push(Transformation::_ReportCountOligos(Box::new(
            reports::_ReportCountOligos::new(
                *report_no,
                count_oligos,
                config.count_oligos_segment.clone(),
            ),
        )));
    }
    if let Some(tag_histograms) = config.tag_histograms.as_ref() {
        for tag_name in tag_histograms {
            res.push(Transformation::_ReportTagHistogram(Box::new(
                reports::_ReportTagHistogram::new(*report_no, tag_name.clone()),
            )));
        }
    }
    *report_no += 1;
}

impl Config {
    /// There are transformations that we need to expand right away,
    /// so we can accurately check the tag stuff
    fn expand_transformations(&mut self) {
        let mut expanded_transforms = Vec::new();
        let mut res_report_labels = Vec::new();
        let mut report_no = 0;

        for t in self.transform.drain(..) {
            match t {
                Transformation::ExtractRegion(step_config) => {
                    let regions = vec![crate::transformations::RegionDefinition {
                        source: step_config.source,
                        resolved_source: None,
                        start: step_config.start,
                        length: step_config.len,
                        anchor: step_config.anchor,
                    }];
                    expanded_transforms.push(Transformation::ExtractRegions(
                        crate::transformations::extract::Regions {
                            out_label: step_config.out_label,
                            regions,
                            // region_separator: None,
                            output_tag_type: std::sync::OnceLock::new(),
                        },
                    ));
                }

                Transformation::Report(report_config) => {
                    expand_reports(
                        &mut expanded_transforms,
                        &mut res_report_labels,
                        &mut report_no,
                        report_config,
                    );
                }
                other => {
                    expanded_transforms.push(other);
                }
            }
        }
        self.transform = expanded_transforms;
        self.report_labels = res_report_labels;
    }

    #[allow(clippy::too_many_lines)]
    pub fn check(&mut self) -> Result<()> {
        let mut errors = Vec::new();
        self.check_input_segment_definitions(&mut errors);
        if errors.is_empty() {
            //no point in checking them if segment definition is broken
            self.check_output(&mut errors);
            self.check_reports(&mut errors);
            self.check_barcodes(&mut errors);
            self.expand_transformations();
            let tag_names = self.check_transformations(&mut errors);
            self.check_for_any_output(&mut errors);
            self.check_input_format(&mut errors);
            self.check_name_collisions(&mut errors, &tag_names);
        }
        if let Err(e) = self.configure_rapidgzip() {
            errors.push(e);
        };
        self.check_benchmark();

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
                bail!("Multiple errors occurred:\n\n{combined_error}");
            }
        }
        assert!(
            self.input.options.use_rapidgzip.is_some(),
            "use_rapidgzip should have been set during check_input_segment_definitions"
        );

        Ok(())
    }

    /// Check configuration for validation mode (allows missing input files)
    #[allow(clippy::too_many_lines)]
    pub fn check_for_validation(&mut self) -> Result<()> {
        let mut errors = Vec::new();
        self.check_input_segment_definitions(&mut errors);
        if errors.is_empty() {
            //no point in checking them if segment definition is broken
            self.check_output(&mut errors);
            self.check_reports(&mut errors);
            self.check_barcodes(&mut errors);
            let tag_names = self.check_transformations(&mut errors);
            self.check_for_any_output(&mut errors);
            self.check_input_format_for_validation(&mut errors);
            self.check_name_collisions(&mut errors, &tag_names);
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
                bail!("Multiple errors occurred:\n\n{combined_error}");
            }
        }

        Ok(())
    }

    fn check_name_collisions(&self, errors: &mut Vec<anyhow::Error>, tag_names: &[String]) {
        //verify that segment_labels, barcode names, and Tag label don't collide
        let mut names_used: HashSet<String> = HashSet::new();
        //segments
        for segment in self.input.get_segment_order() {
            names_used.insert(segment.clone()); //can't be duplicate, toml parsing would have
            //complained
        }
        //barcodes
        for barcode_name in self.barcodes.keys() {
            if !names_used.insert(barcode_name.clone()) {
                errors.push(anyhow!("Name collision: Barcode name '{barcode_name}' collides with an existing segment label"));
            }
        }
        for tag_name in tag_names {
            if names_used.contains(tag_name) {
                errors.push(anyhow!("Name collision: Tag label '{tag_name}' collides with an existing segment label or barcode name"));
            }
        }
    }

    fn check_input_segment_definitions(&mut self, errors: &mut Vec<anyhow::Error>) {
        // Initialize segments and handle backward compatibility
        if let Err(e) = self.input.init() {
            errors.push(e);
        }
    }

    fn check_input_format(&mut self, errors: &mut Vec<anyhow::Error>) {
        self.check_input_duplicate_files(errors);

        let mut saw_fasta = false;
        let mut saw_bam = false;
        match self
            .input
            .structured
            .as_ref()
            .expect("structured input is set during config parsing")
        {
            StructuredInput::Interleaved { files, .. } => {
                let mut interleaved_format: Option<DetectedInputFormat> = None;
                for filename in files {
                    match io::input::detect_input_format(Path::new(filename)) {
                        Ok(format) => {
                            if let Some(existing) = interleaved_format {
                                if existing != format {
                                    errors.push(anyhow!(
                                        "(input): Interleaved inputs must all have the same format. Found both {existing:?} and {format:?} when reading {filename}."
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
                              "(input): Failed to detect input format for interleaved file '{filename}'."
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
                            match io::input::detect_input_format(Path::new(filename)) {
                                Ok(format) => {
                                    if let Some(existing) = segment_format {
                                        if existing != format {
                                            errors.push(anyhow!(
                                                "(input): Segment '{segment_name}' mixes input formats {existing:?} and {format:?}. Mixing formats like this is not supported."
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
                                          "(input): Failed to detect input format for file '{filename}' in segment '{segment_name}'."
                                      )),
                                  ), */
                            }
                        }
                    }
                }
            }
        }

        if saw_fasta && self.input.options.fasta_fake_quality.is_none() {
            errors.push(anyhow!(
                "[input.options]: 'fasta_fake_quality' must be set when reading FASTA inputs."
            ));
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

        if self.options.block_size % 2 == 1 && self.input.is_interleaved() {
            errors.push(anyhow!(
                "[options]: Block size must be even for interleaved input."
            ));
        }
    }

    /// Check input format for validation mode (skips file existence checks)
    fn check_input_format_for_validation(&mut self, errors: &mut Vec<anyhow::Error>) {
        self.check_input_duplicate_files(errors);

        // In validation mode, we skip format detection since files might not exist
        // Just check the block size constraint
        if self.options.block_size % 2 == 1 && self.input.is_interleaved() {
            errors.push(anyhow!(
                "[options]: Block size must be even for interleaved input."
            ));
        }
    }

    fn check_input_duplicate_files(&mut self, errors: &mut Vec<anyhow::Error>) {
        let mut seen = HashSet::new();
        if !self.options.accept_duplicate_files {
            // Check for duplicate files across all segments
            match self
                .input
                .structured
                .as_ref()
                .expect("structured input is set during config parsing")
            {
                StructuredInput::Interleaved { files, .. } => {
                    for f in files {
                        if !seen.insert(f.clone()) {
                            errors.push(anyhow!(
                                "(input): Repeated filename: {f} (in interleaved input). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                            ));
                        }
                    }
                }
                StructuredInput::Segmented {
                    segment_files,
                    segment_order,
                } => {
                    for segment_name in segment_order {
                        let files = segment_files
                            .get(segment_name)
                            .expect("segment_order keys must exist in segment_files");
                        if files.is_empty() {
                            errors.push(anyhow!(
                                "(input): Segment '{segment_name}' has no files specified.",
                            ));
                        }
                        for f in files {
                            if !seen.insert(f.clone()) {
                                errors.push(anyhow!(
                                    "(input): Repeated filename: {f} (in segment '{segment_name}'). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_transform_segments(&mut self, errors: &mut Vec<anyhow::Error>) {
        // check each transformation, validate labels
        for (step_no, t) in self.transform.iter_mut().enumerate() {
            if let Err(e) = t.validate_segments(&self.input) {
                errors.push(e.context(format!("[Step {step_no} ({t})]")));
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn check_transformations(&mut self, errors: &mut Vec<anyhow::Error>) -> Vec<String> {
        self.check_transform_segments(errors);
        if !errors.is_empty() {
            return Vec::new(); // Can't continue validation if segments are invalid
        }
        let mut tags_available: BTreeMap<String, TagMetadata> = BTreeMap::new();

        for (step_no, t) in self.transform.iter().enumerate() {
            if let Err(e) =
                t.validate_others(&self.input, self.output.as_ref(), &self.transform, step_no)
            {
                errors.push(e.context(format!("[Step {step_no} ({t})]:")));
                continue; // Skip further processing of this transform if validation failed
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

            if t.removes_all_tags() {
                for metadata in tags_available.values_mut() {
                    metadata.used = true;
                }
                tags_available.clear();
            }

            if let Some(tag_names_and_types) = t.uses_tags(&tags_available) {
                for (tag_name, tag_types) in tag_names_and_types {
                    //no need to check if empty, empty will never be present
                    let entry = tags_available.get_mut(&tag_name);
                    match entry {
                        Some(metadata) => {
                            metadata.used = true;
                            if !tag_types
                                .iter()
                                .any(|tag_type| tag_type.compatible(metadata.tag_type))
                            {
                                errors.push(anyhow!  (
                            "[Step {step_no} ({t})]: Tag '{label}' does not provide any of the required tag types {supposed_tag_types:?}. It provides '{actual_tag_type}'.", supposed_tag_types=tag_types, label=tag_name, actual_tag_type=metadata.tag_type ));
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

            if let Some((tag_name, tag_type)) = t.declares_tag_type() {
                if let Err(e) = validate_tag_name(&tag_name) {
                    errors.push(anyhow!("[Step {step_no} ({t})]: {e}"));
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
                        tag_type,
                    },
                );
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

        tags_available.keys().cloned().collect()
    }

    fn check_output(&mut self, errors: &mut Vec<anyhow::Error>) {
        //apply output if set
        if let Some(output) = &mut self.output {
            if output.format == FileFormat::Bam {
                if output.output_hash_uncompressed {
                    errors.push(anyhow!(
                        "(output): Uncompressed hashing is not supported when format = 'bam'. Set output_hash_uncompressed = false (and presumably outptu_hash_compressed=true).",
                    ));
                }
                if output.stdout {
                    errors.push(anyhow!(
                        "(output): format = 'bam' cannot be used together with stdout output.",
                    ));
                }
                if output.compression != CompressionFormat::Uncompressed {
                    errors.push(anyhow!(
                        "(output): Compression cannot be specified when format = 'bam'. Remove the compression setting.",
                    ));
                }
            }
            if output.stdout {
                if output.output.is_some() {
                    errors.push(anyhow!(
                        "(output): Cannot specify both 'stdout' and 'output' options together. You need to use 'interleave' to control which segments to output to stdout"
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
                            "(output): Interleave segment '{segment}' not found in input segments: {valid_segments:?}",
                        ));
                    }
                    if !seen_segments.insert(segment) {
                        errors.push(anyhow!(
                            "(output): Interleave segment '{segment}' is duplicated in interleave order: {valid_segments:?}",
                        ));
                    }
                }
                if interleave_order.len() < 2 && !output.stdout {
                    errors.push(anyhow!(
                        "(output): Interleave order must contain at least two segments to interleave. Got: {interleave_order:?}",
                    ));
                }
                //make sure there's no overlap between interleave and output
                if let Some(output_segments) = output.output.as_ref() {
                    for segment in output_segments {
                        if interleave_order.contains(segment) {
                            errors.push(anyhow!(
                                "(output): Segment '{segment}' cannot be both in 'interleave' and 'output' lists. Interleave: {interleave_order:?}, Output: {output_segments:?}",
                            ));
                        }
                    }
                }
            }

            // Validate compression level for output
            if let Err(e) =
                validate_compression_level_u8(output.compression, output.compression_level)
            {
                errors.push(anyhow!("(output): {e}"));
            }

            if output.ix_separator.contains('/')
                || output.ix_separator.contains('\\')
                || output.ix_separator.contains(':')
            {
                errors.push(anyhow!(
                    "(output): 'ix_separator' must not contain path separators such as '/' or '\\' or ':'."
                ));
            }
            if output.ix_separator.is_empty() {
                errors.push(anyhow!("(output): 'ix_separator' must not be empty."));
            }
            if let Some(chunk_size) = output.chunksize {
                if chunk_size == 0 {
                    errors.push(anyhow!(
                        "(output): 'Chunksize' must be greater than zero when specified."
                    ));
                }
                if output.stdout {
                    errors.push(anyhow!(
                        "(output): 'Chunksize' is not supported when writing to stdout."
                    ));
                }
            }
        }
    }
    fn check_reports(&self, errors: &mut Vec<anyhow::Error>) {
        let report_html = self.output.as_ref().is_some_and(|o| o.report_html);
        let report_json = self.output.as_ref().is_some_and(|o| o.report_json);
        let is_benchmark = self.benchmark.as_ref().is_some_and(|b| b.enable);
        let has_report_transforms = self.transform.iter().any(|t| {
            matches!(t, Transformation::Report { .. })
                | matches!(t, Transformation::_InternalReadCount { .. })
        });

        if has_report_transforms && !(report_html || report_json) && !is_benchmark {
            errors.push(anyhow!(
                "(output): Report step configured, but neither output.report_json nor output.report_html is true. Enable at least one to write report files.",
            ));
        }

        if (report_html || report_json) && !has_report_transforms {
            errors.push(anyhow!("(output): Report (html|json) requested, but no report step in configuration. Either disable the reporting, or add a
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
        let is_benchmark = self.benchmark.as_ref().is_some_and(|b| b.enable);

        if !has_fastq_output && !has_report_output && !has_tag_output && !is_benchmark {
            errors.push(anyhow!(
                "(output): No output files and no reports requested. Nothing to do."
            ));
        }
    }

    fn check_barcodes(&self, errors: &mut Vec<anyhow::Error>) {
        // Check that barcode names are unique across all barcodes sections
        for (section_name, barcodes) in &self.barcodes {
            if let Err(e) = validate_tag_name(section_name) {
                errors.push(e.context("Barcode names must be valid tag names"));
            }
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
                errors.push(anyhow!(
                    "[barcodes.{section_name}]: All barcodes in one section must have the same length. Observed: {lengths:?}.",
                ));
            }

            // Check for overlapping IUPAC barcodes
            if let Err(e) = validate_barcode_disjointness(&barcodes.barcode_to_name) {
                errors.push(anyhow!("[barcodes.{section_name}]: {e}"));
            }
        }
    }
    pub fn get_ix_separator(&self) -> String {
        self.output
            .as_ref()
            .map_or_else(output::default_ix_separator, |x| x.ix_separator.clone())
    }

    /// Enable/disable rapidgzip. defaults to enabled if we can find the binary.
    fn configure_rapidgzip(&mut self) -> Result<()> {
        self.input.options.use_rapidgzip = match self.input.options.use_rapidgzip {
            Some(true) => {
                if crate::io::input::find_rapidgzip_in_path().is_none() {
                    bail!(
                        "Warning: rapidgzip requested but not found in PATH. Make sure you have a rapidgzip binary on your path."
                    );
                }
                Some(true)
            }
            Some(false) => Some(false),
            None => Some(crate::io::input::find_rapidgzip_in_path().is_some()),
        };
        Ok(())
    }

    fn check_benchmark(&mut self) {
        if let Some(benchmark) = &self.benchmark {
            if benchmark.enable {
                // Disable output when benchmark mode is enabled
                self.output = Some(Output {
                    prefix: String::from("benchmark"),
                    suffix: None,
                    format: FileFormat::default(),
                    compression: CompressionFormat::default(),
                    compression_level: None,
                    report_html: false,
                    report_json: false,
                    report_timing: false,
                    stdout: false, // Default to false when creating new output config
                    interleave: None,
                    output: Some(Vec::new()),
                    output_hash_uncompressed: false,
                    output_hash_compressed: false,
                    ix_separator: output::default_ix_separator(),
                    chunksize: None,
                });
            }
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
pub struct Barcodes {
    #[serde(
        deserialize_with = "deser::btreemap_iupac_dna_string_from_string",
        flatten
    )]
    #[schemars(skip)]
    pub barcode_to_name: BTreeMap<BString, String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_tag_name_valid() {
        // Valid tag names
        assert!(validate_tag_name("a").is_ok());
        assert!(validate_tag_name("A").is_ok());
        assert!(validate_tag_name("_").is_ok());
        assert!(validate_tag_name("abc").is_ok());
        assert!(validate_tag_name("ABC").is_ok());
        assert!(validate_tag_name("a123").is_ok());
        assert!(validate_tag_name("A123").is_ok());
        assert!(validate_tag_name("_123").is_ok());
        assert!(validate_tag_name("tag_name").is_ok());
        assert!(validate_tag_name("TagName").is_ok());
        assert!(validate_tag_name("tag123_name").is_ok());
        assert!(validate_tag_name("_private_tag").is_ok());
    }

    #[test]
    fn test_validate_tag_name_invalid() {
        // Invalid tag names
        assert!(validate_tag_name("").is_err());
        assert!(validate_tag_name("123").is_err());
        assert!(validate_tag_name("123abc").is_err());
        assert!(validate_tag_name("tag-name").is_err());
        assert!(validate_tag_name("tag.name").is_err());
        assert!(validate_tag_name("tag name").is_err());
        assert!(validate_tag_name("tag@name").is_err());
        assert!(validate_tag_name("tag/name").is_err());
        assert!(validate_tag_name("tag\\name").is_err());
        assert!(validate_tag_name("tag:name").is_err());
    }

    #[test]
    fn test_validate_segment_label_valid() {
        // Valid segment labels
        assert!(validate_segment_label("a").is_ok());
        assert!(validate_segment_label("A").is_ok());
        assert!(validate_segment_label("1").is_ok());
        assert!(validate_segment_label("_").is_ok());
        assert!(validate_segment_label("abc").is_ok());
        assert!(validate_segment_label("ABC").is_ok());
        assert!(validate_segment_label("123").is_ok());
        assert!(validate_segment_label("a123").is_ok());
        assert!(validate_segment_label("A123").is_ok());
        assert!(validate_segment_label("123abc").is_ok());
        assert!(validate_segment_label("read1").is_ok());
        assert!(validate_segment_label("READ1").is_ok());
        assert!(validate_segment_label("segment_name").is_ok());
        assert!(validate_segment_label("segment123").is_ok());
        assert!(validate_segment_label("_internal").is_ok());
    }

    #[test]
    fn test_validate_segment_label_invalid() {
        // Invalid segment labels
        assert!(validate_segment_label("").is_err());
        assert!(validate_segment_label("segment-name").is_err());
        assert!(validate_segment_label("segment.name").is_err());
        assert!(validate_segment_label("segment name").is_err());
        assert!(validate_segment_label("segment@name").is_err());
        assert!(validate_segment_label("segment/name").is_err());
        assert!(validate_segment_label("segment\\name").is_err());
        assert!(validate_segment_label("segment:name").is_err());
    }
}
