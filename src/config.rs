#![allow(clippy::unnecessary_wraps)] //eserde false positives
#![allow(clippy::struct_excessive_bools)] // output false positive, directly on struct doesn't work
                                          //
use crate::io::{self, DetectedInputFormat};
use crate::transformations::{Step, TagValueType, Transformation};
use anyhow::{anyhow, bail, Result};
use bstr::BString;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

pub mod deser;
mod input;
mod options;
mod output;
mod segments;

pub use crate::io::fileformats::PhredEncoding;
pub use input::{
    validate_compression_level_u8, CompressionFormat, FileFormat, Input, InputOptions,
    StructuredInput,
};
pub use options::Options;
pub use output::{default_ix_separator, Output};
pub use segments::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};

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
                                "(input): Repeated filename: {} (in interleaved input). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
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
                                "(input): Segment '{}' has no files specified.",
                                segment_name
                            ));
                        }
                        for f in files {
                            if !seen.insert(f.clone()) {
                                errors.push(anyhow!(
                                    "(input): Repeated filename: {} (in segment '{}'). Probably not what you want. Set options.accept_duplicate_files = true to ignore.",
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
                            match io::detect_input_format(Path::new(filename)) {
                                Ok(format) => {
                                    if let Some(existing) = segment_format {
                                        if existing != format {
                                            errors.push(anyhow!(
                                                "(input): Segment '{segment_name}' mixes input formats {existing:?} and {format:?}. Use separate segments per format."
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

        if self.options.block_size % 2 == 1 && self.input.is_interleaved() {
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

            if t.removes_all_tags() {
                for metadata in tags_available.values_mut() {
                    metadata.used = true;
                }
                tags_available.clear();
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
                        "(output): Uncompressed hashing is not supported when format = 'bam'. Set output_hash_uncompressed = false.",
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
                            "(output): Interleave segment '{}' not found in input segments: {:?}",
                            segment,
                            valid_segments
                        ));
                    }
                    if !seen_segments.insert(segment) {
                        errors.push(anyhow!(
                            "(output): Interleave segment '{}' is duplicated in interleave order: {:?}",
                            segment,
                            interleave_order
                        ));
                    }
                }
                if interleave_order.len() < 2 && !output.stdout {
                    errors.push(anyhow!(
                        "(output): Interleave order must contain at least two segments to interleave. Got: {:?}",
                        interleave_order
                    ));
                }
                //make sure there's no overlap between interleave and output
                if let Some(output_segments) = output.output.as_ref() {
                    for segment in output_segments {
                        if interleave_order.contains(segment) {
                            errors.push(anyhow!(
                                "(output): Segment '{}' cannot be both in 'interleave' and 'output' lists. Interleave: {:?}, Output: {:?}",
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
                errors.push(anyhow!("(output): {}", e));
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

        if !has_fastq_output && !has_report_output && !has_tag_output {
            errors.push(anyhow!(
                "(output): No output files and no reports requested. Nothing to do."
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

#[derive(eserde::Deserialize, Debug, Clone)]
pub struct Barcodes {
    #[serde(
        deserialize_with = "deser::btreemap_iupac_dna_string_from_string",
        flatten
    )]
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
