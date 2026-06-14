use crate::transformations::prelude::*;
use fastqrab_io::CompressionFormat;
use fastqrab_io::io::output::chunked_writer::{BamSinkOptions, read_bam_reference_sequences};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use toml_pretty_deser::suggest_alternatives;

use super::output_fastq::interleave_present;
use super::{
    RecordOutputDeclSpec, RecordOutputState, build_record_declarations, collect_segment_list,
    sink_config, verify_record_targets,
};
use crate::config::{BamOutputOptions, PartialBamOutputOptions};

/// Write reads to BAM file(s) as a pipeline step.
///
/// Replicates the BAM behaviour of the legacy `[output]` section: per-segment
/// files, interleaved output, chunking and compressed hashing, plus BAM
/// auxiliary-tag export (`bam.tags` / `tag_to_bam_tag`) and reference
/// assignment (`bam.tag_to_reference`).
///
/// The reference sequences and the shared BAM header are resolved at
/// config-verify time (see [`resolve_output_bam`]): `references_from_barcodes`
/// from the named `[barcodes.*]` section, `from_bam` by reading the `@SQ`
/// header of the referenced BAM file. The resolved [`BamSinkOptions`] are
/// stashed on the partial and emitted in `declare_output_files`, so the writer
/// is opened with the correct header. The auxiliary-tag values and the
/// reference assignment per read are applied at write time by the BAM writer.
///
/// `bam.merge_demultiplexed` is carried for round-trip; the actual merge of
/// demultiplexed BAMs lives in the binary crate.
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct OutputBAM {
    /// Override the file suffix (defaults to `bam`).
    #[tpd(default)]
    suffix: Option<String>,

    /// BGZF compression level (0-9).
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    compression_level: Option<u8>,
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    compression_threads: usize,

    /// Segments to write to individual files. Defaults to all input segments.
    #[tpd(default, alias = "segments")]
    output: Option<Vec<String>>,

    /// Segments to interleave into a single file.
    #[tpd(default, alias = "interleaved")]
    interleave: Option<Vec<String>>,

    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    chunksize: Option<usize>,

    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    output_hash_compressed: bool,

    /// BAM-specific options (comment separator, tag exports, reference assignment).
    #[tpd(nested)]
    bam: Option<BamOutputOptions>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    output_state: Option<Arc<Mutex<RecordOutputState>>>,

    /// Resolved BAM sink options (header, reference sequences, tag exports),
    /// computed at config-verify time by [`resolve_output_bam`] and consumed in
    /// `declare_output_files` (via the partial config).
    #[tpd(skip, default)]
    #[schemars(skip)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    resolved_bam_options: Option<BamSinkOptions>,
}

const FORMAT: FileFormat = FileFormat::Bam;

/// The pieces of an `OutputBAM` step needed by the binary crate to merge the
/// per-demultiplex-tag BAM files into one BAM per segment.
pub struct OutputBamMergeInfo {
    /// The fastqrab tag whose value selects the reference (and the demultiplex
    /// label to merge over).
    pub reference_label: String,
    /// Write a BAI index alongside each merged BAM.
    pub index_merged: bool,
    /// File suffix of the per-tag BAM files (e.g. `bam`).
    pub suffix: String,
    /// Output basenames to merge: the per-segment names, or `["interleaved"]`
    /// when interleaving.
    pub segment_names: Vec<String>,
    /// BAM records written per molecule in each merged file: 1 for per-segment
    /// files, the number of interleaved segments for an interleaved file.
    pub records_per_molecule: usize,
}

impl OutputBAM {
    /// Merge parameters for this step, or `None` when `merge_demultiplexed` is
    /// not enabled. Used by the binary crate after processing finishes.
    #[must_use]
    pub fn merge_info(&self) -> Option<OutputBamMergeInfo> {
        let bam = self.bam.as_ref()?;
        if bam.merge_demultiplexed != Some(true) {
            return None;
        }
        let reference_label = bam.tag_to_reference.as_ref()?.tag.clone();
        let suffix = FORMAT.get_suffix(CompressionFormat::Uncompressed, self.suffix.as_ref());
        let (segment_names, records_per_molecule) = match self.interleave.as_ref() {
            Some(interleave) => (vec!["interleaved".to_string()], interleave.len().max(1)),
            None => (self.output.clone().unwrap_or_default(), 1),
        };
        Some(OutputBamMergeInfo {
            reference_label,
            index_merged: bam.index_merged,
            suffix,
            segment_names,
            records_per_molecule,
        })
    }
}

impl VerifyIn<PartialConfig> for PartialOutputBAM {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.compression_threads.or(1);
        self.compression_threads.verify(|threads| {
            if *threads == 0 {
                Err(ValidationFailure::new(
                    "Must not be 0.",
                    Some("'compression_threads' must be greater than zero."),
                ))
            } else {
                Ok(())
            }
        });
        if let Some(Some(_)) = self.compression_level.value {
            let compression = TomlValue::new_ok(CompressionFormat::Uncompressed, 0..0);
            crate::config::validate_compression_level_u8(
                &compression,
                &mut self.compression_level,
                &FORMAT,
            );
        }
        self.chunksize.verify(|chunk_size| {
            if let Some(chunk_size) = chunk_size.as_ref()
                && *chunk_size == 0
            {
                return Err(ValidationFailure::new(
                    "Must not be 0.",
                    Some("'chunksize' must be greater than zero when specified."),
                ));
            }
            Ok(())
        });
        // BAM cannot be written to stdout; pass a throwaway stdout flag.
        let mut stdout = TomlValue::new_ok(false, 0..0);
        verify_record_targets(
            parent,
            &mut self.output,
            &mut self.interleave,
            &mut stdout,
            false,
        );
        Ok(())
    }
}

/// Build the (currently minimal) BAM sink options carried in each declaration.
///
/// Only `comment_separation_char` is applied today; tag export / reference
/// resolution are deferred (see the type-level note).
fn declare_bam_options(comment_separation_char: u8) -> BamSinkOptions {
    let mut opts = BamSinkOptions {
        comment_separation_char,
        tag_to_bam_tags: Vec::new(),
        tag_to_reference: None,
        reference_sequences: Arc::new(Vec::new()),
        shared_header: None,
    };
    opts.build_shared_header();
    opts
}

/// Resolve the full [`BamSinkOptions`] for an `OutputBAM` step at config-verify
/// time and stash them on the partial (read later by `declare_output_files`).
///
/// `barcode_section_refs` maps each `[barcodes.*]` section name to its reference
/// `(name, length)` pairs. Builds the auxiliary-tag export list, resolves the
/// reference sequences (`references_from_barcodes` from the map, `from_bam` by
/// reading the file's `@SQ` header) and the BAM header.
///
/// Validation errors (unknown barcode section, unreadable reference BAM) are
/// reported in place on the offending field.
pub(crate) fn resolve_output_bam(
    partial: &mut PartialOutputBAM,
    barcode_section_refs: &IndexMap<String, Vec<(String, usize)>>,
) {
    let mut comment_separation_char = b' ';
    let mut tag_to_bam_tags: Vec<([u8; 2], String)> = Vec::new();
    let mut tag_to_reference: Option<String> = None;
    let mut reference_sequences: Vec<(String, usize)> = Vec::new();

    if let Some(bam) = partial.bam.value.as_mut().and_then(|x| x.as_mut()) {
        if let Some(sep) = bam.comment_separation_char.as_ref() {
            comment_separation_char = *sep;
        }

        if let Some(map_and_keys) = bam.tag_to_bam_tag.value.as_ref() {
            for (tag_label, bam_tag) in &map_and_keys.map {
                if let Some(bam_tag) = bam_tag.as_ref() {
                    tag_to_bam_tags.push((bam_tag.0, tag_label.to_string()));
                }
            }
        }

        if let Some(tag_to_ref) = bam.tag_to_reference.value.as_mut().and_then(|x| x.as_mut()) {
            if let Some(tag_name) = tag_to_ref.tag.as_ref() {
                tag_to_reference = Some(tag_name.clone());
            }
            if let Some(section) = tag_to_ref
                .references_from_barcodes
                .as_ref()
                .and_then(|o| o.as_ref())
                .cloned()
            {
                if let Some(refs) = barcode_section_refs.get(&section) {
                    reference_sequences = refs.clone();
                } else {
                    let available: Vec<&str> =
                        barcode_section_refs.keys().map(String::as_str).collect();
                    tag_to_ref.references_from_barcodes.help =
                        Some(offer_alternatives(&section, &available));
                    tag_to_ref.references_from_barcodes.state =
                        TomlValueState::new_validation_failed(
                            "Barcode section not found for output.bam.tag_to_reference",
                        );
                }
            } else if let Some(path) = tag_to_ref
                .references_from_bam
                .as_ref()
                .and_then(|o| o.as_ref())
                .cloned()
            {
                match read_bam_reference_sequences(Path::new(&path)) {
                    Ok(refs) => reference_sequences = refs,
                    Err(e) => {
                        tag_to_ref.references_from_bam.help = Some(format!("{e:#}"));
                        tag_to_ref.references_from_bam.state =
                            TomlValueState::new_validation_failed(
                                "Could not read reference sequences from BAM file",
                            );
                    }
                }
            }
        }
    }

    let mut opts = BamSinkOptions {
        comment_separation_char,
        tag_to_bam_tags,
        tag_to_reference,
        reference_sequences: Arc::new(reference_sequences),
        shared_header: None,
    };
    opts.build_shared_header();
    partial.resolved_bam_options = Some(Some(opts));
}

/// Validate `bam.merge_demultiplexed` on an `OutputBAM` step.
///
/// `available_demultiplex_labels` are the `in_label`s of the (lookup-mode)
/// `Demultiplex` steps in the pipeline. Mirrors the legacy
/// `verify_merge_demultiplexed`, minus the `format = 'bam'` check (an
/// `OutputBAM` step is always BAM). Errors are reported on `merge_demultiplexed`.
pub(crate) fn verify_output_bam_merge(
    partial: &mut PartialOutputBAM,
    available_demultiplex_labels: &[String],
) {
    let interleave_empty = match partial.interleave.as_ref() {
        Some(Some(x)) => x.is_empty(),
        Some(None) => true,
        None => false,
    };
    let output_empty = matches!(partial.output.as_ref(), Some(Some(x)) if x.is_empty());
    let output_span = partial.output.span();

    let Some(bam) = partial.bam.value.as_mut().and_then(|x| x.as_mut()) else {
        return;
    };
    if !matches!(bam.merge_demultiplexed.as_ref(), Some(Some(true))) {
        return;
    }

    if output_empty && interleave_empty {
        bam.merge_demultiplexed.state = TomlValueState::Custom {
            spans: vec![
                (
                    bam.merge_demultiplexed.span(),
                    "Incompatible with empty outputs".to_string(),
                ),
                (output_span, "These output segments are empty".to_string()),
            ],
        };
        bam.merge_demultiplexed.help = Some(
            "Either remove 'merge_demultiplexed' or specify either output segments or interleaved output.".to_string(),
        );
    }

    match bam.tag_to_reference.as_ref() {
        Some(Some(tag_to_reference)) => {
            if let Some(ref_label) = tag_to_reference.tag.as_ref()
                && !available_demultiplex_labels.contains(ref_label)
            {
                bam.merge_demultiplexed.state = TomlValueState::new_validation_failed(format!(
                    "No Demultiplex step found that had in_label = {ref_label}",
                ));
                if available_demultiplex_labels.is_empty() {
                    bam.merge_demultiplexed.help = Some(
                        "No suitable Demultiplex step found. Make sure you have a Demultiplex step with lookup_mode = 'lookup' and an in_label that matches output.bam.tag_to_reference.tag.".to_string(),
                    );
                } else {
                    bam.merge_demultiplexed.help = Some(format!(
                        "Either add a Demultiplex step or reuse one of the following: {}",
                        suggest_alternatives("", available_demultiplex_labels)
                    ));
                }
            }
        }
        Some(None) => {
            bam.merge_demultiplexed.state = TomlValueState::new_validation_failed(
                "merge_demultiplexed requires tag_to_reference to be set.",
            );
            bam.merge_demultiplexed.help = Some(
                "Either remove 'merge_demultiplexed' or set 'tag_to_reference' to specify how to assign reads to BAM references for merging."
                    .to_string(),
            );
        }
        None => {} // cov:excl-line
    }
}

impl TagUser for PartialTaggedVariant<PartialOutputBAM> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        let inner = self.toml_value.value.as_mut()?;
        let mut used_tags: Vec<Option<UsedTag<'_>>> = Vec::new();
        let mut used_barcodes: HashSet<TagLabel> = HashSet::new();

        if let Some(bam) = inner.bam.value.as_mut().and_then(|x| x.as_mut()) {
            // Each fastqrab tag exported to a BAM auxiliary tag must exist. The
            // map keys carry the raw label text (`TomlValue<String>`); the
            // parsed `TagLabel`s are the keys of `map`.
            if let Some(map_and_keys) = bam.tag_to_bam_tag.value.as_mut() {
                for (tv_key, tag_label) in map_and_keys.keys.iter_mut().zip(map_and_keys.map.keys())
                {
                    if tv_key.is_ok() {
                        used_tags.push(Some(UsedTag {
                            name: tag_label.clone(),
                            accepted_tag_types: ANY_TAG_TYPE,
                            toml_source: Rc::new(RefCell::new((
                                &mut tv_key.state,
                                &mut tv_key.help,
                            ))),
                            further_help: None,
                        }));
                    }
                }
            }
            // The reference-selecting tag must exist; `from_barcodes` names a
            // barcode section we then count as "used".
            if let Some(tag_to_ref) = bam.tag_to_reference.value.as_mut().and_then(|x| x.as_mut()) {
                if let Some(section) = tag_to_ref
                    .references_from_barcodes
                    .as_ref()
                    .and_then(|o| o.as_ref())
                {
                    used_barcodes.insert(TagLabel::Normal(section.clone()));
                }
                if let Some(tag_name) = tag_to_ref.tag.as_ref() {
                    let name = TagLabel::Normal(tag_name.clone());
                    used_tags.push(Some(UsedTag {
                        name,
                        accepted_tag_types: ANY_TAG_TYPE,
                        toml_source: Rc::new(RefCell::new((
                            &mut tag_to_ref.tag.state,
                            &mut tag_to_ref.tag.help,
                        ))),
                        further_help: None,
                    }));
                }
            }
        }

        Some(TagUsageInfo {
            used_tags,
            used_barcodes,
            ..Default::default()
        })
    }

    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let inner = self
            .toml_value
            .value
            .as_ref()
            .expect("declare_output_files called without successful verification");

        let comment_separation_char = inner
            .bam
            .as_ref()
            .and_then(|b| b.as_ref())
            .and_then(|b| b.comment_separation_char.as_ref().copied())
            .unwrap_or(b' ');

        // Reference sequences / header / tag exports are resolved earlier by
        // `resolve_output_bam`; fall back to a minimal header if (somehow) absent.
        let bam_options = inner
            .resolved_bam_options
            .clone()
            .flatten()
            .unwrap_or_else(|| declare_bam_options(comment_separation_char));

        let segments = collect_segment_list(&inner.output);
        let interleave =
            interleave_present(&inner.interleave).then(|| collect_segment_list(&inner.interleave));
        let suffix = FORMAT.get_suffix(
            CompressionFormat::Uncompressed,
            inner.suffix.as_ref().and_then(|x| x.as_ref()),
        );
        let spec = RecordOutputDeclSpec {
            format: FORMAT,
            suffix,
            segments: &segments,
            interleave: interleave.as_deref(),
            stdout: false,
            sink_config: sink_config(
                CompressionFormat::Uncompressed,
                inner
                    .compression_level
                    .as_ref()
                    .and_then(|x| x.as_ref())
                    .copied(),
                inner.compression_threads.as_ref().copied().unwrap_or(1),
                false,
                *inner.output_hash_compressed.unwrap_ref(),
            ),
            chunksize: inner.chunksize.as_ref().and_then(|x| x.as_ref()).copied(),
            bam_options: Some(bam_options),
            span: self.toml_value.span(),
        };
        build_record_declarations(&spec)
    }
}

impl Step for OutputBAM {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let segments = self.output.clone().unwrap_or_default();
        let state = RecordOutputState::from_step_output_files(
            &mut output_files,
            &input_info.segment_order,
            &segments,
            self.interleave.as_deref(),
            false,
        );
        self.output_state = Some(Arc::new(Mutex::new(state)));
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        self.output_state
            .as_ref()
            .expect("output_state should have been set in init")
            .lock()
            .expect("lock poisoned")
            .write_block(&block, demultiplex_info)
            .context("Error in OutputBAM stage")?;
        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        self.output_state
            .as_ref()
            .expect("output_state should have been set in init")
            .lock()
            .expect("lock poisoned")
            .finish()?;
        Ok(None)
    }
}
