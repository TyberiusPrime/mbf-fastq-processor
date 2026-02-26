#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)]
use crate::config::options::default_block_size;
// output false positive, directly on struct doesn't work
//
use crate::io::{self, DetectedInputFormat};
use crate::transformations::{PartialTransformation, Step, TagValueType, Transformation};
use anyhow::{Result, anyhow, bail};
use bstr::BString;
use indexmap::IndexMap;
use schemars::JsonSchema;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use toml_pretty_deser::prelude::*;

pub mod deser;
mod input;
pub mod options;
mod output;
mod segments;

use crate::{dna, get_number_of_cores};
pub use deser::validate_tag_name;
pub use input::{
    CompressionFormat, FileFormat, Input, InputOptions, PartialInput, PartialInputOptions,
    STDIN_MAGIC_PATH, StructuredInput,
};
pub use io::fileformats::PhredEncoding;
pub use options::{Options, PartialOptions};
pub use output::{Output, PartialOutput, validate_compression_level_u8};
pub use segments::{
    ResolvedSourceAll, ResolvedSourceNoAll, SegmentIndex, SegmentIndexOrAll, SegmentOrNameIndex,
    ValidateSegment,
};

#[derive(Debug)]
pub struct TagMetadata {
    pub used: bool,
    pub declared_at_step: usize,
    pub declared_by: String,
    pub tag_type: TagValueType,
}

pub fn config_from_string(toml: &str) -> Result<Config, DeserError<PartialConfig>> {
    Config::tpd_from_toml(toml, FieldMatchMode::AnyCase, VecMode::SingleOk)
}

/// Validates that a segment label conforms to the pattern [a-zA-Z0-9_]+
/// (one or more alphanumeric characters or underscores)
pub fn validate_segment_label(
    label: &str,
    match_mode: toml_pretty_deser::prelude::FieldMatchMode,
) -> Result<()> {
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
                "Segment label must contain only letters, numbers, and underscores (^[a-zA-Z0-9_]+$), found '{ch}'.",
            );
        }
    }
    for prohibited in &[
        "fasta_fake_quality",
        "bam_include_mapped",
        "bam_include_unmapped",
        "read_comment_character",
        "use_rapidgzip",
        "build_rapidgzip_index",
        "threads_per_segment",
        "tpd_field_match_mode",
    ] {
        if match_mode.matches(label, *prohibited) {
            bail!(
                "'{prohibited}' is not allowed as a segment label, as it could be confused with an existing option name or an internal. Please choose a different segment name, or prefix in with 'options.' if you meant the option."
            );
        }
    }

    Ok(())
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[tpd(no_verify)]
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

#[derive(JsonSchema)]
#[tpd(root)]
#[derive(Debug)]
pub struct Config {
    /// The input configuration
    #[tpd(nested)]
    pub input: Input,
    #[tpd(nested)]
    pub output: Option<Output>,

    //barcodes must happen before transforms
    #[schemars(with = "BTreeMap<String, Barcodes>")]
    #[tpd(nested)]
    pub barcodes: Option<IndexMap<String, Barcodes>>,

    #[tpd(alias = "step")]
    #[tpd(nested)]
    pub transform: Vec<Transformation>,

    #[tpd(nested)]
    pub options: Options,

    #[tpd(nested)]
    pub benchmark: Option<Benchmark>,
}

#[derive(Debug)]
pub struct CheckedConfig {
    pub input: Input,
    pub output: Option<Output>,
    pub stages: Vec<Stage>,
    pub options: Options,
    pub barcodes: IndexMap<String, Barcodes>,
    pub benchmark: Option<Benchmark>,

    pub report_labels: Vec<String>,
}

impl VerifyIn<TPDRoot> for PartialConfig {
    fn verify(&mut self, parent: &TPDRoot) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        if !self.input.is_ok() {
            //we can't check transforms if the input def has failed,
            //they'de all have not set their segments/sources
            self.transform = TomlValue::new_ok(Vec::new(), 0..0);
        }
        self.transform.or_default();
        self.options.or_with(|| PartialOptions {
            threads: TomlValue::new_ok(None, 0..0),
            max_blocks_in_flight: TomlValue::new_ok(None, 0..0),
            block_size: TomlValue::new_ok(default_block_size(), 0..0),
            buffer_size: TomlValue::new_ok(options::default_buffer_size(), 0..0),
            output_buffer_size: TomlValue::new_ok(options::default_output_buffer_size(), 0..0),
            accept_duplicate_files: TomlValue::new_ok(false, 0..0),
            spot_check_read_pairing: TomlValue::new_ok(
                options::default_spot_check_read_pairing(),
                0..0,
            ),
            debug_failures: TomlValue::new_ok(
                options::PartialFailureOptions {
                    fail_output_after_bytes: TomlValue::new_ok(None, 0..0),
                    fail_output_error: TomlValue::new_ok(None, 0..0),
                    fail_output_raw_os_code: TomlValue::new_ok(None, 0..0),
                    tpd_field_match_mode: parent.tpd_field_match_mode,
                },
                0..0,
            ),
            tpd_field_match_mode: parent.tpd_field_match_mode,
        });
        self.verify_reports();
        self.verify_barcodes();
        self.verify_benchmark_molecule_count();
        self.disable_output_on_benchmark();
        self.verify_for_any_output();
        self.configure_rapid_gzip();
        self.verify_head_rapidgzip_conflict();
        Ok(())
    }
}

impl PartialConfig {
    fn verify_reports(&mut self) {
        let report_html = self
            .output
            .as_ref()
            .and_then(|x| x.as_ref())
            .and_then(|x| x.report_html.as_ref())
            .is_some_and(|o| *o);
        let report_json = self
            .output
            .as_ref()
            .and_then(|x| x.as_ref())
            .and_then(|x| x.report_json.as_ref())
            .is_some_and(|o| *o);
        let is_benchmark = self
            .benchmark
            .as_ref()
            .and_then(|x| x.as_ref())
            .and_then(|x| x.enable.as_ref())
            .is_some_and(|o| *o);
        let mut report_transform = self.transform.as_mut().and_then(|x| {
            x.iter_mut().find(|t| {
                matches!(
                    t.as_ref(),
                    Some(PartialTransformation::Report { .. })
                        | Some(PartialTransformation::_InternalReadCount { .. })
                )
            })
        });

        if let Some(report_transform) = &mut report_transform
            && !(report_html || report_json)
            && !is_benchmark
        {
            let spans = vec![
                (
                    self.output.span(),
                    "Add report_json | report_html here?".to_string(),
                ),
                (
                    report_transform.span(),
                    "Report but no output.report_html | report_json".to_string(),
                ),
            ];

            report_transform.state = TomlValueState::Custom { spans };
            report_transform.help =
                Some("Either remove the report, or enable it's output.".to_string());
        } else if (report_html || report_json) && report_transform.is_none() {
            let mut spans = Vec::new();
            if let Some(tv_report_html) = self
                .output
                .as_ref()
                .and_then(|x| x.as_ref())
                .map(|x| &x.report_html)
            {
                if let Some(true) = tv_report_html.as_ref() {
                    spans.push((tv_report_html.span(), "Set to true?".to_string()));
                }
            }
            if let Some(tv_report_json) = self
                .output
                .as_ref()
                .and_then(|x| x.as_ref())
                .map(|x| &x.report_json)
            {
                if let Some(true) = tv_report_json.as_ref() {
                    spans.push((tv_report_json.span(), "Set to true?".to_string()));
                }
            }
            if spans.is_empty() {
                spans.push((
                    self.output.span(),
                    "Missing report_html | report_json = true?".to_string(),
                ));
            }
            self.output.state = TomlValueState::Custom { spans };
            self.output.help =
                Some("No report step, but report output requested.\nRemove/disable report_html & report_json, or add in a report step.".to_string());
        }
    }

    fn verify_benchmark_molecule_count(&mut self) {
        if let Some(Some(benchmark)) = self.benchmark.as_mut() {
            benchmark.molecule_count.verify(|v| {
                if *v == 0 {
                    Err(ValidationFailure::new(
                        "molecule_count must be > 0",
                        Some("Set to a positive integer."),
                    ))
                } else {
                    Ok(())
                }
            });
        }
    }

    fn verify_head_rapidgzip_conflict(&mut self) {
        let build_rapidgzip_index = self
            .input
            .as_ref()
            .and_then(|i| i.options.as_ref())
            .and_then(|o| o.build_rapidgzip_index.as_ref())
            .and_then(|x| x.as_ref())
            .copied()
            .unwrap_or(false);
        if !build_rapidgzip_index {
            return;
        }
        let rapidgzip_span = self
            .input
            .as_ref()
            .and_then(|i| i.options.as_ref())
            .map(|o| o.build_rapidgzip_index.span())
            .unwrap_or_default();
        let mut head_transform = self.transform.as_mut().and_then(|x| {
            x.iter_mut()
                .find(|t| matches!(t.as_ref(), Some(PartialTransformation::Head(..))))
        });
        if let Some(head_tv) = &mut head_transform {
            let spans = vec![
                (
                    head_tv.span(),
                    "This Head transform conflicts with build_rapidgzip_index".to_string(),
                ),
                (
                    rapidgzip_span,
                    "build_rapidgzip_index = true set here".to_string(),
                ),
            ];
            head_tv.state = TomlValueState::Custom { spans };
            head_tv.help = Some(
                "build_rapidgzip_index and Head cannot be used together (index would not be created). Set build_rapidgzip_index to false.".to_string(),
            );
        }
    }

    fn verify_for_any_output(&mut self) {
        let has_fastq_output = self
            .output
            .as_ref()
            .and_then(|x| x.as_ref())
            .map(|o| {
                o.stdout.as_ref().copied().unwrap_or(false)
                    || o.output
                        .as_ref()
                        .map(|inner| inner.as_ref().map_or(true, |v| !v.is_empty()))
                        .unwrap_or(true)
                    || o.interleave
                        .as_ref()
                        .and_then(|inner| inner.as_ref())
                        .map(|v| !v.is_empty())
                        .unwrap_or(false)
            })
            .unwrap_or(false);
        let has_report_output = self
            .output
            .as_ref()
            .and_then(|x| x.as_ref())
            .map(|o| {
                o.report_html.as_ref().copied().unwrap_or(false)
                    || o.report_json.as_ref().copied().unwrap_or(false)
            })
            .unwrap_or(false);
        let has_tag_output = self
            .transform
            .as_ref()
            .map(|transforms| {
                transforms.iter().any(|t| {
                    matches!(
                        t.as_ref(),
                        Some(PartialTransformation::StoreTagInFastQ(..))
                            | Some(PartialTransformation::StoreTagsInTable(..))
                            | Some(PartialTransformation::Inspect(..))
                    )
                })
            })
            .unwrap_or(false);
        let is_benchmark = self
            .benchmark
            .as_ref()
            .and_then(|x| x.as_ref())
            .and_then(|x| x.enable.as_ref())
            .copied()
            .unwrap_or(false);
        let output_ok = self.output.is_ok();
        if !has_fastq_output && !has_report_output && !has_tag_output && !is_benchmark && output_ok
        {
            self.output.state = TomlValueState::ValidationFailed {
                message: "No output files and no reports requested. Nothing to do.".to_string(),
            };
            self.output.help = Some(
                "Add an [output] section with output files or reports, or use a benchmark configuration.".to_string(),
            );
        }
    }

    fn verify_barcodes(&mut self) {
        // Check that barcode names are unique across all barcodes sections
        if let Some(Some(barcodes)) = self.barcodes.as_mut() {
            for tv_section_name in barcodes.keys.iter_mut() {
                if let Some(section_name) = tv_section_name.as_ref()
                    && let Err(e) = validate_tag_name(section_name)
                {
                    tv_section_name.state = TomlValueState::new_validation_failed("Invalid value");
                    tv_section_name.help = Some(e.to_string());
                }
            }
            for (_section_name, tv_barcodes) in barcodes.map.iter_mut() {
                if let Some(barcodes) = tv_barcodes.as_mut()
                    && let Some(barcodes) = barcodes.barcode_to_name.as_mut()
                {
                    for key in barcodes.keys.iter_mut() {
                        if let Some(key_str) = key.as_ref() {
                            if !crate::dna::all_iupac_or_underscore(key_str.as_bytes()) {
                                key.state = TomlValueState::new_validation_failed("Invalid value");
                                key.help = Some("Barcode contains non-IUPAC / spacer characters. Only A,C,G,T, IUPAC ambiguity codes and '_' are allowed.".to_string());
                            }
                        }
                    }
                    if barcodes.map.is_empty() {
                        tv_barcodes.state = TomlValueState::new_validation_failed(
                            "At least one barcode mapping must be provided",
                        );
                        tv_barcodes.help = Some(
                            "Add at least one barcode mapping (DNA='name') under this section"
                                .to_string(),
                        );
                        break;
                    }
                    // assert that barcodes have all the same length

                    let mut lengths: HashSet<usize> = HashSet::new();
                    for (iupac_key, barcode_name) in barcodes.map.iter_mut() {
                        if let Some(barcode_name_str) = barcode_name.as_ref() {
                            if barcode_name_str == "no-barcode" {
                                barcode_name.state =
                                    TomlValueState::new_validation_failed("Must not be no-barcode");
                                barcode_name.help =
                                    Some("Choose a different name for your barcode".to_string());
                            }
                            lengths.insert(iupac_key.len());
                        }
                    }
                    if lengths.len() > 1 {
                        tv_barcodes.state =
                            TomlValueState::new_validation_failed("Barcodes of different lengths");
                        let mut lengths: Vec<_> = lengths.into_iter().collect();
                        lengths.sort();
                        tv_barcodes.help = Some(format!(
                            "All barcodes in one section must have the same length. Observed lengths: {lengths:?}"
                        ));
                        break;
                    }
                    // Check for overlapping IUPAC barcodes
                    validate_barcode_disjointness(barcodes);
                }
            }
        }
    }

    fn disable_output_on_benchmark(&mut self) {
        if let Some(Some(benchmark)) = &self.benchmark.as_ref()
            && let Some(true) = benchmark.enable.as_ref()
        {
            // Disable output when benchmark mode is enabled
            self.output = TomlValue::new_ok(
                Some(PartialOutput {
                    prefix: TomlValue::new_ok(String::from("benchmark"), 0..0),
                    suffix: TomlValue::new_ok(None, 0..0),
                    format: TomlValue::new_ok(FileFormat::default(), 0..0),
                    compression: TomlValue::new_ok(CompressionFormat::default(), 0..0),
                    compression_level: TomlValue::new_ok(None, 0..0),
                    compression_threads: TomlValue::new_ok(None, 0..0),
                    report_html: TomlValue::new_ok(false, 0..0),
                    report_json: TomlValue::new_ok(false, 0..0),
                    report_timing: TomlValue::new_ok(false, 0..0),
                    stdout: TomlValue::new_ok(false, 0..0),
                    interleave: TomlValue::new_ok(None, 0..0),
                    output: TomlValue::new_ok(Some(Vec::new()), 0..0),
                    output_hash_uncompressed: TomlValue::new_ok(false, 0..0),
                    output_hash_compressed: TomlValue::new_ok(false, 0..0),
                    ix_separator: TomlValue::new_ok(output::default_ix_separator(), 0..0),
                    chunksize: TomlValue::new_ok(None, 0..0),
                    tpd_field_match_mode: self.tpd_field_match_mode,
                }),
                0..0,
            );
        }
    }

    /// Enable/disable rapidgzip. defaults to enabled if we can find the binary.
    fn configure_rapid_gzip(&mut self) {
        if let Some(input) = self.input.as_mut()
            && let Some(options) = input.options.as_mut()
        {
            options.use_rapidgzip.value = match options.use_rapidgzip.as_ref() {
                Some(Some(true)) => {
                    if crate::io::input::find_rapidgzip_in_path().is_none() {
                        options.use_rapidgzip.state = TomlValueState::ValidationFailed {
                            message: "rapidgzip requested but not found in PATH".to_string(),
                        };
                        options.use_rapidgzip.help = Some(
                            "Make sure you have a rapidgzip binary on your path, or set use_rapidgzip to false (or leave off for auto-detect).".to_string(),
                        );
                    }
                    Some(Some(true))
                }
                Some(Some(false)) => Some(Some(false)),
                Some(None) => Some(Some(crate::io::input::find_rapidgzip_in_path().is_some())),
                None => None, //other error
            }
        }
    }
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
    push_existing(Transformation::Report(config.clone())); // for validation. 
    // We remove it again later on. Transformation::Expand
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
            reports::_ReportCountOligos::new(*report_no, count_oligos, config.count_oligos_segment),
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
    #[allow(clippy::too_many_lines)]
    fn expand_transformations(&mut self) -> Vec<String> {
        let expanded_transforms = RefCell::new(Vec::new());
        let mut res_report_labels = Vec::new();
        let mut report_no = 0;
        let mut push_existing = |t: Transformation| expanded_transforms.borrow_mut().push(t);
        let mut push_new = |t: Transformation| {
            expanded_transforms.borrow_mut().push(t);
        };

        self.expand_spot_checks(&mut push_new);

        for t in self.transform.drain(..) {
            match t {
                Transformation::ExtractRegion(step_config) => {
                    let regions = vec![crate::transformations::RegionDefinition {
                        source: step_config.source,
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
                            segment: SegmentIndexOrAll::All,
                        },
                    ));
                    push_new(t);
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
        self.transform = expanded_transforms.into_inner();
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
            .iter()
            .any(|step| matches!(step, Transformation::ValidateName(_)));
        let has_spot_check = self
            .transform
            .iter()
            .any(|step| matches!(step, Transformation::ValidateReadPairing(_)));
        let is_benchmark = self.benchmark.as_ref().is_some_and(|b| b.enable);

        if !has_validate_name && !has_spot_check && !is_benchmark {
            push_new(Transformation::ValidateReadPairing(
                crate::transformations::validation::ValidateReadPairing::default(),
            ));
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn check(self) -> Result<CheckedConfig> {
        self.inner_check(true)
    }

    fn inner_check(mut self, check_input_files_exist: bool) -> Result<CheckedConfig> {
        let mut errors = Vec::new();
        let mut stages = None;

        //no point in checking them if segment definition is broken
        //self.check_output(&mut errors);
        let report_labels = Some(self.expand_transformations());
        if errors.is_empty() {
            let (tag_names, stages_) = self.check_transformations(&mut errors);
            //self.transfrom is now empty, the trafos have been expanded into stepsk.
            let stages_ = stages_;
            assert!(self.transform.is_empty());
            self.check_name_collisions(&mut errors, &tag_names);
            if check_input_files_exist {
                let input_formats_observed = self.check_input_format(&mut errors);
                self.configure_multithreading(&input_formats_observed);
            } else {
                self.check_input_format_for_validation(&mut errors);
            }
            stages = Some(stages_);
        }

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
            barcodes: self.barcodes.unwrap_or_default(),
            benchmark: self.benchmark,
            report_labels: report_labels.expect("Set above"),
        })
    }

    /// Check configuration for validation mode (allows missing input files)
    #[allow(clippy::too_many_lines)]
    pub fn check_for_validation(self) -> Result<CheckedConfig> {
        self.inner_check(false)
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
        if let Some(barcodes) = self.barcodes.as_ref() {
            for barcode_name in barcodes.keys() {
                barcode_names_used.insert(barcode_name.clone());
                if segment_names_used.contains(barcode_name) {
                    errors.push(anyhow!("Name collision: Barcode name '{barcode_name}' collides with an existing segment label"));
                }
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

        match &self.input.structured {
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

        InputFormatsObserved {
            fastq: saw_fastq,
            fasta: saw_fasta,
            bam: saw_bam,
            gzip: saw_gzip,
        }
    }

    /// Check input format for validation mode (skips file existence checks)
    fn check_input_format_for_validation(&mut self, errors: &mut Vec<anyhow::Error>) {
        self.check_input_duplicate_files(errors);
    }

    fn check_input_duplicate_files(&mut self, errors: &mut Vec<anyhow::Error>) {
        let mut seen = HashSet::new();
        if !self.options.accept_duplicate_files {
            // Check for duplicate files across all segments
            match &self.input.structured {
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

    #[allow(clippy::too_many_lines)]
    fn check_transformations(
        &mut self,
        errors: &mut Vec<anyhow::Error>,
    ) -> (Vec<String>, Vec<Stage>) {
        let mut tags_available: IndexMap<String, TagMetadata> = IndexMap::new();
        let mut allowed_tags_per_stage = Vec::new();

        for (step_no, t) in self.transform.iter().enumerate() {
            if let Err(e) =
                t.validate_others(&self.input, self.output.as_ref(), &self.transform, step_no)
            {
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
                tags_available.shift_remove(&tag_name);
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
                for (tag_name, tag_types) in &tag_names_and_types {
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
            } else if t.must_see_all_tags() {
                tags_available.keys().cloned().collect()
            } else {
                Vec::new()
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
        let stages: Vec<Stage> = self
            .transform
            .drain(..)
            .zip(allowed_tags_per_stage)
            .filter(|(t, _)| !matches!(t, Transformation::Report { .. }))
            .map(|(t, tags)| Stage {
                transformation: t,
                allowed_tags: tags,
            })
            .collect();

        (tags_available.keys().cloned().collect(), stages)
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

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Barcodes {
    // #[serde(
    //     deserialize_with = "deser::btreemap_iupac_dna_string_from_string",
    //     flatten
    // )]
    #[schemars(with = "BTreeMap<String, String>")]
    #[tpd(absorb_remaining)]
    pub barcode_to_name: IndexMap<BString, String>,
}

impl VerifyIn<PartialConfig> for PartialBarcodes {
    fn verify(&mut self, _parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.barcode_to_name.verify_keys(|key| {
            if dna::all_iupac_or_underscore(key.as_bytes()) {
                Ok(())
            } else {
                Err(ValidationFailure::new(
                        "Invalid IUPAC (uppercase only)",
                        Some("See https://en.wikipedia.org/wiki/International_Union_of_Pure_and_Applied_Chemistry#Amino_acid_and_nucleotide_base_codes")
                ))}
        });
        Ok(())
    }
}
//
/// Validate that IUPAC barcodes are disjoint (don't overlap in their accepted sequences)

#[allow(clippy::collapsible_if)]
#[mutants::skip] // yeah, modifying to for j in (i * 1) will still 'work', just perform more checks
fn validate_barcode_disjointness(barcodes: &mut MapAndKeys<BString, String>) {
    // First pass: collect all overlapping pairs without mutating anything.
    // We must not assign while iterating because one barcode can overlap multiple others
    // (e.g. NNNN overlaps both ATCG and RYRN); assigning in-loop would overwrite earlier results.
    let mut overlapping_pairs: Vec<(usize, usize, Vec<(std::ops::Range<usize>, String)>)> =
        Vec::new();

    for i in 0..barcodes.keys.len() {
        for j in (i + 1)..barcodes.keys.len() {
            if let Some(dna_a) = barcodes.keys[i].value.as_ref()
            && let Some(dna_b) = barcodes.keys[j].value.as_ref()
            && let Some(barcode_name_a) = barcodes.map.get(bstr::BStr::new(dna_a)).and_then(|x| x.as_ref())
            && let Some(barcode_name_b) = barcodes.map.get(bstr::BStr::new(dna_b)).and_then(|x| x.as_ref())
            && barcode_name_a != barcode_name_b
            && crate::dna::iupac_overlapping(dna_a.as_bytes(), dna_b.as_bytes())
            {
                let spans = vec![
                    (barcodes.keys[i].span(), format!("Overlaps with {dna_b}")),
                    (barcodes.keys[j].span(), format!("Overlaps with {dna_a}")),
                ];
                overlapping_pairs.push((i, j, spans));
            }
        }
    }

    // Second pass: assign each pair's error to the first barcode in the pair that hasn't
    // already been used as an error anchor, so every overlap gets its own error entry.
    let mut assigned: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for (i, j, spans) in overlapping_pairs {
        let error_idx = if !assigned.contains(&i) { i } else { j };
        barcodes.keys[error_idx].state = TomlValueState::Custom { spans };
        barcodes.keys[error_idx].help =
            Some("IUPAC patterns overlap, but lead to different barcodes.".to_string());
        assigned.insert(error_idx);
    }
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
        let f = toml_pretty_deser::FieldMatchMode::Exact;
        assert!(validate_segment_label("a",f).is_ok());
        assert!(validate_segment_label("A",f).is_ok());
        assert!(validate_segment_label("_",f).is_ok());
        assert!(validate_segment_label("abc",f).is_ok());
        assert!(validate_segment_label("ABC",f).is_ok());
        assert!(validate_segment_label("123",f).is_err());
        assert!(validate_segment_label("a123",f).is_ok());
        assert!(validate_segment_label("A123",f).is_ok());
        assert!(validate_segment_label("123abc",f).is_err());
        assert!(validate_segment_label("read1",f).is_ok());
        assert!(validate_segment_label("READ1",f).is_ok());
        assert!(validate_segment_label("segment_name",f).is_ok());
        assert!(validate_segment_label("segment123",f).is_ok());
        assert!(validate_segment_label("_internal",f).is_ok());
    }

    #[test]
    fn test_validate_segment_label_invalid() {
        // Invalid segment labels
        let f = toml_pretty_deser::FieldMatchMode::Exact;
        assert!(validate_segment_label("",f).is_err());
        assert!(validate_segment_label("1",f).is_err());
        assert!(validate_segment_label("segment-name",f).is_err());
        assert!(validate_segment_label("segment.name",f).is_err());
        assert!(validate_segment_label("segment name",f).is_err());
        assert!(validate_segment_label("segment@name",f).is_err());
        assert!(validate_segment_label("segment/name",f).is_err());
        assert!(validate_segment_label("segment\\name",f).is_err());
        assert!(validate_segment_label("segment:name",f).is_err());
        assert!(validate_segment_label("fasta_fake_quality",f).is_err());
        assert!(validate_segment_label("bam_include_mapped",f).is_err());
        assert!(validate_segment_label("bam_include_unmapped",f).is_err());
        assert!(validate_segment_label("read_comment_character",f).is_err());
        assert!(validate_segment_label("use_rapidgzip",f).is_err());
        assert!(validate_segment_label("build_rapidgzip_index",f).is_err());
        assert!(validate_segment_label("threads_per_segment",f).is_err());
        assert!(validate_segment_label("tpd_field_match_mode",f).is_err());

        let f = toml_pretty_deser::FieldMatchMode::AnyCase;
        assert!(validate_segment_label("FaSTA___FAKE-QUALITY",f).is_err());
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
