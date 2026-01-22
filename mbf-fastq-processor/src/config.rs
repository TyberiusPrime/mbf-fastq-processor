#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
//
use crate::io::{self, DetectedInputFormat};
use crate::transformations::{Step, TagValueType, Transformation};
use anyhow::{Result, anyhow, bail};
use bstr::BString;
use schemars::JsonSchema;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod deser;
mod input;
pub mod options;
mod output;
mod segments;

use crate::get_number_of_cores;
pub use input::{
    CompressionFormat, FileFormat, Input, InputOptions, STDIN_MAGIC_PATH, StructuredInput,
    validate_compression_level_u8,
};
pub use io::fileformats::PhredEncoding;
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

    for (forbidden, reason) in &[
        ("ReadName", "the index column in StoreTagsInTable"),
        ("read_no", "read numbering in EvalExpression"),
    ] {
        if tag_name == *forbidden {
            // because that's what we store in the output tables as
            // column 0
            bail!(
                "Reserved tag label '{forbidden}' cannot be used as a tag label. This name is reserved for {reason}. Please choose a different tag name."
            );
        }
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
        if i == 0 && !ch.is_ascii_alphabetic() && ch != '_' {
            bail!("Segment label must start with a letter or underscore (^[a-zA-Z_]), got '{ch}'",);
        }
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            bail!(
                "Segment label must contain only letters, numbers, and underscores (^[a-zA-Z0-9_]+$), found '{ch}' at position {i}",
            );
        }
    }

    Ok(())
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    /// Enable benchmark mode
    #[serde(default)]
    pub enable: bool,

    /// Number of molecules to process in benchmark mode
    pub molecule_count: usize,
}

#[derive(eserde::Deserialize, Debug, JsonSchema, Default)]
#[allow(dead_code)] //we currently only use gzip for multi thread considerations, but set them all
struct InputFormatsObserved {
    fastq: bool,
    fasta: bool,
    bam: bool,
    gzip: bool,
}

#[derive(Debug)]
pub struct Stage {
    pub transformation: Transformation,
    pub allowed_tags: Vec<String>,
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
    pub transform: Option<Vec<Transformation>>,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    pub barcodes: BTreeMap<String, Barcodes>,
    #[serde(default)]
    pub benchmark: Option<Benchmark>,
}

#[derive(Debug)]
pub struct CheckedConfig {
    pub input: Input,
    pub output: Option<Output>,
    pub stages: Vec<Stage>,
    pub options: Options,
    pub barcodes: BTreeMap<String, Barcodes>,
    pub benchmark: Option<Benchmark>,

    pub report_labels: Vec<String>,
}

#[allow(clippy::used_underscore_items)]
fn expand_reports<F: FnMut(Transformation), G: FnMut(Transformation)>(
    mut push_new: F,
    mut push_existing: G,
    res_report_labels: &mut Vec<String>,
    report_no: &mut usize,
    config: crate::transformations::reports::Report,
) {
    use crate::transformations::prelude::DemultiplexedData;
    use crate::transformations::reports;
    push_existing(Transformation::Report(config.clone())); // for validation. We remove it again later
    // on.
    // Transformation::Expand
    res_report_labels.push(config.name);
    if config.count {
        push_new(Transformation::_ReportCount(Box::new(
            reports::_ReportCount::new(*report_no),
        )));
    }
    if config.length_distribution {
        push_new(Transformation::_ReportLengthDistribution(Box::new(
            reports::_ReportLengthDistribution::new(*report_no),
        )));
    }
    if config.duplicate_count_per_read {
        push_new(Transformation::_ReportDuplicateCount(Box::new(
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
        push_new(Transformation::_ReportDuplicateFragmentCount(Box::new(
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
        push_new(Transformation::_ReportBaseStatisticsPart1(Box::new(
            reports::_ReportBaseStatisticsPart1::new(*report_no),
        )));
        push_new(Transformation::_ReportBaseStatisticsPart2(Box::new(
            reports::_ReportBaseStatisticsPart2::new(*report_no),
        )));
    }

    if let Some(count_oligos) = config.count_oligos.as_ref() {
        push_new(Transformation::_ReportCountOligos(Box::new(
            reports::_ReportCountOligos::new(
                *report_no,
                count_oligos,
                config.count_oligos_segment.clone(),
            ),
        )));
    }
    if let Some(tag_histograms) = config.tag_histograms.as_ref() {
        for tag_name in tag_histograms {
            push_new(Transformation::_ReportTagHistogram(Box::new(
                reports::_ReportTagHistogram::new(*report_no, tag_name.clone()),
            )));
        }
    }
    *report_no += 1;
}

impl Config {
    /// There are transformations that we need to expand right away,
    /// so we can accurately check the names
    fn expand_transformations(&mut self, errors: &mut Vec<anyhow::Error>) -> Vec<String> {
        let expanded_transforms = RefCell::new(Vec::new());
        let mut res_report_labels = Vec::new();
        let mut report_no = 0;
        let mut push_existing = |t: Transformation| expanded_transforms.borrow_mut().push(t);
        let mut push_new = |mut t: Transformation| {
            let step_no = expanded_transforms.borrow().len() + 1;
            if let Err(e) = t.validate_segments(&self.input) {
                errors.push(e.context(format!("[Step {step_no} (after expansion) ({t})]")));
            }
            expanded_transforms.borrow_mut().push(t);
        };

        self.expand_spot_checks(&mut push_new);

        for t in self
            .transform
            .take()
            .expect(".transform has to be still valid in expand")
            .drain(..)
        {
            match t {
                Transformation::ExtractRegion(step_config) => {
                    let regions = vec![crate::transformations::RegionDefinition {
                        source: step_config.source,
                        resolved_source: None,
                        start: step_config.start,
                        length: step_config.len,
                        anchor: step_config.anchor,
                    }];
                    push_new(Transformation::ExtractRegions(
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
                        &mut push_new,
                        &mut push_existing,
                        &mut res_report_labels,
                        &mut report_no,
                        report_config,
                    );
                }

                Transformation::_InternalReadCount(step_config) => {
                    res_report_labels.push(step_config.out_label.clone());
                    let step_config: Box<_> =
                        Box::new(crate::transformations::_InternalReadCount::new(
                            step_config.out_label,
                            report_no,
                        ));
                    report_no += 1;
                    push_new(Transformation::_InternalReadCount(step_config));
                }
                Transformation::CalcGCContent(step_config) => {
                    push_new(Transformation::CalcBaseContent(
                        step_config.into_base_content(),
                    ));
                }
                Transformation::CalcNCount(config) => {
                    push_new(Transformation::CalcBaseContent(config.into_base_content()));
                }
                Transformation::FilterEmpty(step_config) => {
                    // Replace FilterEmpty with CalcLength + FilterByNumericTag
                    let length_tag_label =
                        format!("_internal_length_{}", expanded_transforms.borrow().len());
                    push_new(Transformation::CalcLength(
                        crate::transformations::calc::Length {
                            out_label: length_tag_label.clone(),
                            segment: step_config.segment,
                            segment_index: step_config.segment_index,
                        },
                    ));
                    push_new(Transformation::FilterByNumericTag(
                        crate::transformations::filters::ByNumericTag {
                            in_label: length_tag_label,
                            min_value: Some(1.0), // Non-empty means length >= 1
                            max_value: None,
                            keep_or_remove: crate::transformations::KeepOrRemove::Keep,
                        },
                    ));
                }
                Transformation::ConvertQuality(ref step_config) => {
                    //implies a check beforehand
                    push_new(Transformation::ValidateQuality(
                        crate::transformations::validation::ValidateQuality {
                            encoding: step_config.from,
                            segment: SegmentOrAll("all".to_string()),
                            segment_index: Some(SegmentIndexOrAll::All),
                        },
                    ));
                    push_new(t);
                }
                Transformation::ValidateName(step_config) => {
                    let mut replacement =
                        crate::transformations::validation::SpotCheckReadPairing::default();
                    replacement.sample_stride = 1;
                    replacement.readname_end_char = step_config.readname_end_char;
                    push_new(Transformation::SpotCheckReadPairing(replacement));
                }
                Transformation::Lowercase(step_config) => {
                    push_new(Transformation::_ChangeCase(
                        crate::transformations::edits::_ChangeCase::new(
                            step_config.target,
                            crate::transformations::edits::CaseType::Lower,
                            step_config.if_tag,
                        ),
                    ));
                }
                Transformation::Uppercase(step_config) => {
                    push_new(Transformation::_ChangeCase(
                        crate::transformations::edits::_ChangeCase::new(
                            step_config.target,
                            crate::transformations::edits::CaseType::Upper,
                            step_config.if_tag,
                        ),
                    ));
                }

                other => {
                    push_existing(other);
                }
            }
        }
        self.transform = Some(expanded_transforms.into_inner());
        res_report_labels
    }

    fn expand_spot_checks<F: FnMut(Transformation)>(&self, mut push_new: F) {
        if !self.options.spot_check_read_pairing {
            return;
        }
        if self.input.segment_count() <= 1 {
            return;
        }

        let has_validate_name = self
            .transform
            .as_ref()
            .expect(".transform has to be still valid in expand_spot_checks")
            .iter()
            .any(|step| matches!(step, Transformation::ValidateName(_)));
        let has_spot_check = self
            .transform
            .as_ref()
            .expect(".transform has to be still valid in expand_spot_checks")
            .iter()
            .any(|step| matches!(step, Transformation::SpotCheckReadPairing(_)));
        let is_benchmark = self.benchmark.as_ref().is_some_and(|b| b.enable);

        if !has_validate_name && !has_spot_check && !is_benchmark {
            push_new(Transformation::SpotCheckReadPairing(
                crate::transformations::validation::SpotCheckReadPairing::default(),
            ));
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn check(self) -> Result<CheckedConfig> {
        self._check(true)
    }

    fn _check(mut self, check_input_files_exist: bool) -> Result<CheckedConfig> {
        let mut errors = Vec::new();
        self.check_input_segment_definitions(&mut errors);
        let mut stages = None;
        let mut report_labels = None;
        if self.transform.is_none() {
            // configuring no transformations is fine.
            // But since we're using an option to represent
            // 'no more .transform after this point in the checking'
            // we have to do this manual init
            self.transform = Some(Vec::new());
        }
        if errors.is_empty() {
            //no point in checking them if segment definition is broken
            self.check_output(&mut errors);
            assert!(self.transform.is_some());
            self.check_reports(&mut errors);
            assert!(self.transform.is_some());
            self.check_barcodes(&mut errors);
            self.check_transform_segments(&mut errors);
            report_labels = Some(self.expand_transformations(&mut errors));
            if errors.is_empty() {
                let (tag_names, stages_) = self.check_transformations(&mut errors);
                //self.transfrom is now None, the trafos have been expanded into stepsk.
                let stages_ = stages_;
                assert!(self.transform.is_none());
                self.check_name_collisions(&mut errors, &tag_names);
                self.check_for_any_output(&stages_, &mut errors);
                if check_input_files_exist {
                    let input_formats_observed = self.check_input_format(&mut errors);
                    self.configure_multithreading(&input_formats_observed);
                } else {
                    self.check_input_format_for_validation(&mut errors);
                }
                self.check_head_rapidgzip_conflict(&stages_, &mut errors);
                if let Err(e) = self.configure_rapidgzip() {
                    errors.push(e);
                }
                stages = Some(stages_);
            }
        }
        self.check_benchmark(&mut errors);

        // Return collected errors if any
        if !errors.is_empty() {
            // For multiple errors, format them cleanly
            let combined_error = errors
                .into_iter()
                .map(|e| format!("{e:?}"))
                .collect::<Vec<_>>()
                .join("\n\n---------\n\n");
            bail!("Multiple errors occurred:\n\n{combined_error}");
        }
        assert!(
            self.input.options.use_rapidgzip.is_some(),
            "use_rapidgzip should have been set during check_input_segment_definitions"
        );

        Ok(CheckedConfig {
            input: self.input,
            output: self.output,
            stages: stages.expect("Set above"),
            options: self.options,
            barcodes: self.barcodes,
            benchmark: self.benchmark,
            report_labels: report_labels.expect("Set above"),
        })
    }

    /// Check configuration for validation mode (allows missing input files)
    #[allow(clippy::too_many_lines)]
    pub fn check_for_validation(self) -> Result<CheckedConfig> {
        self._check(false)
    }

    fn check_name_collisions(&self, errors: &mut Vec<anyhow::Error>, tag_names: &[String]) {
        //verify that segment_labels, barcode names, and Tag label don't collide
        let mut segment_names_used: HashSet<String> = HashSet::new();
        //segments
        for segment in self.input.get_segment_order() {
            segment_names_used.insert(segment.clone()); //can't be duplicate, TOML parsing would have
            //complained
        }
        let mut barcode_names_used: HashSet<String> = HashSet::new();
        //barcodes
        for barcode_name in self.barcodes.keys() {
            barcode_names_used.insert(barcode_name.clone());
            if segment_names_used.contains(barcode_name) {
                errors.push(anyhow!("Name collision: Barcode name '{barcode_name}' collides with an existing segment label"));
            }
        }
        for tag_name in tag_names {
            if segment_names_used.contains(tag_name) {
                errors.push(anyhow!(
                    "Name collision: Tag label '{tag_name}' collides with an existing segment label"
                ));
            }
            if barcode_names_used.contains(tag_name) {
                errors.push(anyhow!(
                    "Name collision: Tag label '{tag_name}' collides with an existing barcode name"
                ));
            }
        }
    }

    fn check_head_rapidgzip_conflict(&self, stages: &Vec<Stage>, errors: &mut Vec<anyhow::Error>) {
        let has_head_transform = stages
            .iter()
            .any(|stage| matches!(stage.transformation, Transformation::Head { .. }));
        if has_head_transform && self.input.options.build_rapidgzip_index == Some(true) {
            errors.push(anyhow!(
                "input.options.build_rapidgzip_index and Head can not be used together (index would not be created). Set `input.options.build_rapidgzip_index` to false"
            ));
        }
    }

    fn check_input_segment_definitions(&mut self, errors: &mut Vec<anyhow::Error>) {
        // Initialize segments and handle backward compatibility
        if let Err(e) = self.input.init() {
            errors.push(e);
        }
    }

    #[allow(clippy::similar_names)]
    #[allow(clippy::too_many_lines)]
    #[mutants::skip] // saw_gzip is only necessary for multi threading, and that's not being
    // observed
    fn check_input_format(&mut self, errors: &mut Vec<anyhow::Error>) -> InputFormatsObserved {
        self.check_input_duplicate_files(errors);

        let mut saw_fasta = false;
        let mut saw_bam = false;
        let mut saw_fastq = false;
        let mut saw_gzip = false;

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
                        Ok((format, compression_format)) => {
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
                                DetectedInputFormat::Fastq => {
                                    saw_fastq = true;
                                    if compression_format == CompressionFormat::Gzip {
                                        saw_gzip = true;
                                    }
                                }
                                DetectedInputFormat::Fasta => {
                                    saw_fasta = true;
                                    if compression_format == CompressionFormat::Gzip {
                                        saw_gzip = true;
                                    }
                                }
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
                                Ok((format, compression_format)) => {
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
                                        DetectedInputFormat::Fastq => {
                                            saw_fastq = true;
                                            if compression_format == CompressionFormat::Gzip {
                                                saw_gzip = true;
                                            }
                                        }
                                        DetectedInputFormat::Fasta => {
                                            saw_fasta = true;
                                            if compression_format == CompressionFormat::Gzip {
                                                saw_gzip = true;
                                            }
                                        }
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

        self.check_blocksize(errors);
        InputFormatsObserved {
            fastq: saw_fastq,
            fasta: saw_fasta,
            bam: saw_bam,
            gzip: saw_gzip,
        }
    }

    fn check_blocksize(&self, errors: &mut Vec<anyhow::Error>) {
        if self.options.block_size == 0 {
            errors.push(anyhow!(
                "[options]: Block size must be > 0. Set to a positive integer."
            ));
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
        self.check_blocksize(errors);
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
                                "(input): Repeated filename: \"{f}\" (in interleaved input). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
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
                                    "(input): Repeated filename: \"{f}\" (in segment '{segment_name}'). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_transform_segments(&mut self, errors: &mut Vec<anyhow::Error>) {
        // check each transformations (before & after expansion), validate labels
        for (step_no, t) in self
            .transform
            .as_mut()
            .expect(".transform has to be still valid in check_transform_segments")
            .iter_mut()
            .enumerate()
        {
            if let Err(e) = t.validate_segments(&self.input) {
                errors.push(e.context(format!("[Step {step_no} ({t})]")));
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn check_transformations(
        &mut self,
        errors: &mut Vec<anyhow::Error>,
    ) -> (Vec<String>, Vec<Stage>) {
        let mut tags_available: BTreeMap<String, TagMetadata> = BTreeMap::new();
        let mut allowed_tags_per_stage = Vec::new();

        for (step_no, t) in self
            .transform
            .as_ref()
            .expect(".transform has to be still valid in check_transform_segments")
            .iter()
            .enumerate()
        {
            if let Err(e) = t.validate_others(
                &self.input,
                self.output.as_ref(),
                self.transform
                    .as_ref()
                    .expect(".transform has to be still valid in check_transformations"),
                step_no,
            ) {
                errors.push(e.context(format!("[Step {step_no} ({t})]:")));
                continue; // Skip further processing of this transform if validation failed
            }

            for tag_name in t.removes_tags() {
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

            if t.removes_all_tags() {
                for metadata in tags_available.values_mut() {
                    metadata.used = true;
                }
                tags_available.clear();
            }

            let tags_here: Vec<String> = if let Some(tag_names_and_types) =
                t.uses_tags(&tags_available)
            {
                for (tag_name, tag_types) in tag_names_and_types.iter() {
                    //no need to check if empty, empty will never be present
                    let entry = tags_available.get_mut(tag_name);
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
                if t.must_see_all_tags() {
                    tags_available.keys().cloned().collect()
                } else {
                    tag_names_and_types
                        .into_iter()
                        .map(|(name, _)| name)
                        .collect()
                }
            } else {
                if t.must_see_all_tags() {
                    tags_available.keys().cloned().collect()
                } else {
                    Vec::new()
                }
            };

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
            allowed_tags_per_stage.push(tags_here);
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
        let transforms = self.transform.take();
        let stages: Vec<Stage> = transforms
            .expect(".transform has to be still valid in check_transformations")
            .into_iter()
            .zip(allowed_tags_per_stage)
            .filter(|(t, _)| !matches!(t, Transformation::Report { .. }))
            .map(|(t, tags)| Stage {
                transformation: t,
                allowed_tags: tags,
            })
            .collect();

        (tags_available.keys().cloned().collect(), stages)
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
            let valid_segments: HashSet<&String> = self.input.get_segment_order().iter().collect();

            if let Some(output_segments) = output.output.as_ref() {
                let mut seen_segments = HashSet::new();
                for segment in output_segments {
                    if !valid_segments.contains(segment) {
                        errors.push(anyhow!(
                            "(output.output): Segment '{segment}' not found in input segments: {valid_segments:?}",
                        ));
                    }
                    if !seen_segments.insert(segment) {
                        errors.push(anyhow!(
                            "(output): Segment '{segment}' is duplicated in interleave order: {valid_segments:?}",
                        ));
                    }
                }
            }

            if let Some(interleave_order) = output.interleave.as_ref() {
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
        let has_report_transforms = self
            .transform
            .as_ref()
            .expect(".transform has to be still valid in check_reports")
            .iter()
            .any(|t| {
                matches!(
                    t,
                    Transformation::Report { .. } | Transformation::_InternalReadCount { .. }
                )
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

    fn check_for_any_output(&self, stages: &Vec<Stage>, errors: &mut Vec<anyhow::Error>) {
        let has_fastq_output = self.output.as_ref().is_some_and(|o| {
            o.stdout
                || o.output.as_ref().is_none_or(|o| !o.is_empty())
                || o.interleave.as_ref().is_some_and(|i| !i.is_empty())
        });
        let has_report_output = self
            .output
            .as_ref()
            .is_some_and(|o| o.report_html || o.report_json);
        let has_tag_output = stages.iter().any(|stage| {
            matches!(
                stage.transformation,
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

    #[mutants::skip] // yeah, no rapidgzip doesn't change the result
    fn configure_multithreading(&mut self, input_formats_observed: &InputFormatsObserved) {
        let segment_count = self.input.parser_count();
        let can_multicore_input = input_formats_observed.gzip;
        // self.input_formats_observed.saw_bam as of 2025-12-16, multi core bam isn't faster. I
        // mean the user can enable it by setting threads_per_segment > 1, but by default we
        // choose one core

        let can_multicore_compression = self
            .output
            .as_ref()
            .is_some_and(|o| matches!(o.compression, CompressionFormat::Gzip));
        let (thread_count, input_threads_per_segment, output_threads) = calculate_thread_counts(
            self.options.threads,
            self.input.options.threads_per_segment,
            self.output.as_ref().and_then(|x| x.compression_threads),
            segment_count,
            get_number_of_cores(),
            can_multicore_input,
            can_multicore_compression,
        );
        self.options.threads = Some(thread_count);
        self.input.options.threads_per_segment = Some(input_threads_per_segment);
        if let Some(output) = &mut self.output {
            output.compression_threads = Some(output_threads);
        }

        //rapidgzip single core is slower than regular gzip
        if self.input.options.threads_per_segment.expect("Set before") == 1 
            // if user requests an index, run rapidgzip anyway
            && !self.input.options.build_rapidgzip_index.unwrap_or(false)
            // if the user explicitly requested rapidgzip, then do don't disable it.
            && self.input.options.use_rapidgzip != Some(true)
        {
            // otherwise, we can fall back
            self.input.options.use_rapidgzip = Some(false);
        }
    }

    fn check_benchmark(&mut self, errors: &mut Vec<anyhow::Error>) {
        if let Some(benchmark) = &self.benchmark
            && benchmark.enable
        {
            if benchmark.molecule_count == 0 {
                errors.push(anyhow!(
                    "Benchmark needs a molecule_count > 0. Set to a positive integer."
                ));
            }
            // Disable output when benchmark mode is enabled
            self.output = Some(Output {
                prefix: String::from("benchmark"),
                suffix: None,
                format: FileFormat::default(),
                compression: CompressionFormat::default(),
                compression_level: None,
                compression_threads: None,
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

impl CheckedConfig {
    pub fn get_ix_separator(&self) -> String {
        self.output
            .as_ref()
            .map_or_else(output::default_ix_separator, |x| x.ix_separator.clone())
    }
}

fn calculate_thread_counts(
    step_thread_count: Option<usize>,
    threads_per_segment: Option<usize>,
    compression_threads: Option<usize>,
    segment_count: usize,
    cpu_count: usize,
    can_multicore_decompression: bool,
    can_multicore_compression: bool,
) -> (usize, usize, usize) {
    let threads_per_segment = if can_multicore_decompression {
        threads_per_segment
    } else {
        Some(1)
    };
    let compression_threads = compression_threads.unwrap_or_else(|| {
        if can_multicore_compression {
            let half = cpu_count / 2;
            half.min(5)
        } else {
            1
        }
    });

    match (step_thread_count, threads_per_segment) {
        (Some(step_thread_count), Some(threads_per_segment)) => {
            (step_thread_count, threads_per_segment, compression_threads)
            //keep whatever the user set.
        }
        (None, Some(threads_per_segment)) => (
            //all remaining cores into steps
            cpu_count
                .saturating_sub(threads_per_segment * segment_count)
                .max(1),
            threads_per_segment,
            compression_threads,
        ),
        (Some(thread_count), None) => {
            //all remaining cores into parsing
            let per_segment = (cpu_count.saturating_sub(thread_count) / segment_count).max(1);
            (thread_count, per_segment, compression_threads)
        }
        (None, None) => {
            let half = cpu_count / 2;
            //our benchmarks says the sweet spot is somewhere around 5 threads per segment
            let threads_per_segment = (half / segment_count).clamp(1, 5);
            (
                //if we rounded down, or had way more cores, we will use more threads per steps
                cpu_count
                    .saturating_sub(threads_per_segment * segment_count)
                    .max(1),
                threads_per_segment,
                compression_threads,
            )
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
#[mutants::skip] // yeah, modifying to for j in (i * 1) will still 'work', just perform more checks
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
        assert!(validate_tag_name("len_123").is_err());
        assert!(validate_tag_name("len_shu").is_err());
        assert!(validate_tag_name("ReadName").is_err());
        assert!(validate_tag_name("read_no").is_err());
    }

    #[test]
    fn test_validate_segment_label_valid() {
        // Valid segment labels
        assert!(validate_segment_label("a").is_ok());
        assert!(validate_segment_label("A").is_ok());
        assert!(validate_segment_label("_").is_ok());
        assert!(validate_segment_label("abc").is_ok());
        assert!(validate_segment_label("ABC").is_ok());
        assert!(validate_segment_label("123").is_err());
        assert!(validate_segment_label("a123").is_ok());
        assert!(validate_segment_label("A123").is_ok());
        assert!(validate_segment_label("123abc").is_err());
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
        assert!(validate_segment_label("1").is_err());
        assert!(validate_segment_label("segment-name").is_err());
        assert!(validate_segment_label("segment.name").is_err());
        assert!(validate_segment_label("segment name").is_err());
        assert!(validate_segment_label("segment@name").is_err());
        assert!(validate_segment_label("segment/name").is_err());
        assert!(validate_segment_label("segment\\name").is_err());
        assert!(validate_segment_label("segment:name").is_err());
    }

    #[test]
    fn test_calculate_thread_counts() {
        // Test various combinations of inputs
        assert_eq!(
            calculate_thread_counts(Some(8), Some(2), None, 4, 16, true, false),
            (8, 2, 1)
        );
        assert_eq!(
            calculate_thread_counts(Some(8), Some(2), None, 40, 1, true, false),
            (8, 2, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, Some(2), None, 4, 16, true, false),
            (8, 2, 1)
        );
        assert_eq!(
            calculate_thread_counts(Some(8), None, None, 4, 16, true, false),
            (8, 2, 1)
        );
        assert_eq!(
            calculate_thread_counts(Some(9), None, None, 4, 16, true, false),
            (9, 1, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, None, None, 4, 16, true, false),
            (8, 2, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, None, None, 2, 16, true, false),
            (8, 4, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, None, None, 1, 16, true, false),
            (11, 5, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, None, None, 1, 16, false, false),
            (15, 1, 1)
        );
        assert_eq!(
            calculate_thread_counts(None, None, None, 1, 16, false, true),
            (15, 1, 5)
        );

        assert_eq!(
            calculate_thread_counts(None, None, None, 1, 8, false, true),
            (7, 1, 4)
        );
    }
}
