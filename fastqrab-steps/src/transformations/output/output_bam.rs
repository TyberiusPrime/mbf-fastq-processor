use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_io::CompressionFormat;
use fastqrab_io::io::output::chunked_writer::{BamSinkOptions, SharedBamHeader};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::PathBuf;
use std::rc::Rc;
use toml_pretty_deser::prelude::*;

use super::output_fastq::interleave_present;
use super::{
    RecordOutputDeclSpec, RecordOutputState, build_record_declarations, collect_segment_list,
    sink_config, verify_chunk_size, verify_record_targets, verify_suffix,
};

#[must_use]
pub fn default_bam_comment_separation_char() -> u8 {
    b' '
}

/// A validated two-character BAM auxiliary tag name (e.g. `"BC"`).
///
/// Only ASCII alphanumeric characters are accepted.
#[derive(Clone, Debug, JsonSchema)]
#[schemars(with = "String")]
pub struct BamTag(pub [u8; 2]);

impl TryFrom<&str> for BamTag {
    type Error = String;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return Err(format!(
                "BAM tag must be exactly 2 characters; got '{s}' ({} chars). \
                 BAM auxiliary tag names are exactly 2 ASCII alphanumeric characters.",
                bytes.len()
            ));
        }
        if !bytes.iter().all(|&b| b.is_ascii_alphanumeric()) {
            return Err(format!(
                "BAM tag must be 2 alphanumeric ASCII characters; got '{s}'. \
                 Only [A-Za-z0-9] are allowed.",
            ));
        }
        Ok(BamTag([bytes[0], bytes[1]]))
    }
}

toml_pretty_deser::impl_visitor_for_try_from_str!(BamTag, "Invalid BAM tag");
// Custom scalar (its own visitor); appears behind a `#[tpd(nested)]` map, so it
// needs a no-op alias-tree impl. It declares no aliases of its own.

/// Source for reference sequences used by `tag_to_reference` (Feature B).
///
/// Exactly one of `barcodes` or `from_bam` must be set.
#[tpd(no_verify)]
#[derive(Clone, Debug, JsonSchema)]
pub struct TagToReference {
    /// The fastqrab tag label whose value selects the reference sequence name.
    pub tag: String,

    /// Name of a `[barcodes.<name>]` section whose keys become reference names.
    #[tpd(default, alias = "from_barcodes")]
    pub references_from_barcodes: Option<String>,

    /// Path to a BAM file whose `@SQ` header lines define the reference sequences.
    #[tpd(default, alias = "from_bam", alias = "template")]
    pub references_from_bam: Option<String>,
}

/// Alias so the `#[tpd]` macro can find the "partial" type for `BamTag`
/// (which is its own visitor – no separate Partial struct is generated).
pub type PartialBamTag = BamTag;

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

    // BAM-specific options (comment separator, tag exports, reference assignment).
    /// Character used to split read names into a BAM name field and a `CO` auxiliary tag.
    /// Defaults to `' '` (space).  Reads whose names contain this character are split; the
    /// part after the character is placed in a `CO` tag so it can exceed the 254-byte limit.
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    #[schemars(with = "Option<u8>")]
    pub comment_separation_char: u8,

    /// Map of fastqrab tag labels to BAM auxiliary tag names
    ///
    /// Each key is a fastqrab tag label; each value is the two-character BAM
    /// auxiliary tag name to write (e.g. `BC`).
    #[tpd(nested, alias = "tags")]
    #[schemars(with = "Option<std::collections::BTreeMap<String, String>>")]
    pub tag_to_bam_tag: IndexMap<TagLabel, BamTag>,

    /// Export a fastqrab tag value as the BAM reference name
    #[tpd(nested)]
    pub tag_to_reference: Option<TagToReference>,

    #[tpd(default)]
    pub merge_demultiplexed: Option<bool>,

    /// Write a BAI index alongside each merged BAM file (default: true).
    pub index_merged: bool,

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
        if self.merge_demultiplexed != Some(true) {
            return None;
        }
        let reference_label = self.tag_to_reference.as_ref()?.tag.clone();
        let suffix = FORMAT.get_suffix(CompressionFormat::Uncompressed, self.suffix.as_ref());
        let (segment_names, records_per_molecule) = match self.interleave.as_ref() {
            Some(interleave) => (vec!["interleaved".to_string()], interleave.len().max(1)),
            None => (self.output.clone().unwrap_or_default(), 1),
        };
        Some(OutputBamMergeInfo {
            reference_label,
            index_merged: self.index_merged,
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
        self.compression_level.verify(|level| {
            if let Some(&level) = level.as_ref()
                && level > 9
            {
                Err(ValidationFailure::new(
                    "Invalid compression level specified for BAM output",
                    Some("Valid range is 0-9 for BAM (and our compressor)"),
                ))
            } else {
                Ok(())
            }
        });
        self.chunksize
            .verify(|chunk_size| verify_chunk_size(chunk_size, &TomlValue::new_ok(false, 0..0)));
        self.suffix.verify(verify_suffix);
        // BAM cannot be written to stdout; pass a throwaway stdout flag.
        let mut stdout = TomlValue::new_ok(false, 0..0);
        verify_record_targets(
            parent,
            &mut self.output,
            &mut self.interleave,
            &mut stdout,
            false,
        );

        self.comment_separation_char
            .or_with(default_bam_comment_separation_char);
        self.tag_to_bam_tag
            .or_with(|| toml_pretty_deser::MapAndKeys {
                map: indexmap::IndexMap::new(),
                keys: vec![],
            });

        // Each BAM auxiliary tag may only be written once: reject two fastqrab
        // tags mapping to the same two-letter BAM tag.
        if let Some(map_and_keys) = self.tag_to_bam_tag.as_mut() {
            let mut seen_bam_tags: IndexMap<[u8; 2], std::ops::Range<usize>> = IndexMap::new();
            for bam_tag in map_and_keys.map.values_mut() {
                let span = bam_tag.span().clone();
                if let Some(bam_tag_value) = bam_tag.as_mut() {
                    if let Some(other_span) = seen_bam_tags.insert(bam_tag_value.0, span) {
                        bam_tag.state = TomlValueState::Custom {
                            spans: vec![
                                (bam_tag.span(), "Repeated, 2nd use".to_string()),
                                (other_span, "Repeated, 1st use".to_string()),
                            ],
                        };
                        bam_tag.help = Some(
                            "BAM tags must be distinct, \
                            can not write two tags into one BAM tag. Rename either one"
                                .to_string(),
                        );
                    } else {
                        if bam_tag_value.0 == [b'C', b'O'] {
                            bam_tag.state = TomlValueState::ValidationFailed {
                                message: "Conflicts with read name / comment splitting".to_string(),
                            };
                            bam_tag.help = Some(
                            "BAM tag 'CO' is reserved for comments (split read names at `comment_separation_char`), \
                            can not export another tag to 'CO'.\n\
                            Rename this tag."
                                .to_string(),
                        );
                        }
                    }
                }
            }
        }

        // Validate tag_to_reference: exactly one of barcodes or from_bam must be set.
        //
        if let Some(Some(tag_to_ref)) = self.tag_to_reference.as_mut() {
            let has_barcodes = tag_to_ref
                .references_from_barcodes
                .as_ref()
                .and_then(|x| x.as_ref())
                .is_some();
            let has_from_bam = tag_to_ref
                .references_from_bam
                .as_ref()
                .and_then(|x| x.as_ref())
                .is_some();
            if !has_barcodes && !has_from_bam {
                tag_to_ref.references_from_barcodes.state = TomlValueState::new_validation_failed(
                    "Either 'reference_from_barcodes' or 'reference_from_bam' must be specified",
                );
                tag_to_ref.references_from_barcodes.help = Some(
                    "Set 'reference_from_barcodes' to a barcode section name, or 'references_from_bam' to a BAM file path."
                        .to_string(),
                );
            } else if has_barcodes && has_from_bam {
                tag_to_ref.references_from_bam.state = TomlValueState::Custom {
                    spans: vec![
                        (
                            tag_to_ref.references_from_barcodes.span(),
                            "Conflicts with from_bam".to_string(),
                        ),
                        (
                            tag_to_ref.references_from_bam.span(),
                            "Conflicts with barcodes".to_string(),
                        ),
                    ],
                };
                tag_to_ref.references_from_bam.help =
                    Some("Set only one of 'barcodes' or 'from_bam'.".to_string());
            }
        }
        self.index_merged.or(true);

        Ok(())
    }
}

/// Build the (currently minimal) BAM sink options carried in each declaration.
///
/// Only `comment_separation_char` is applied today; tag export / reference
/// resolution are deferred (see the type-level note).
fn declare_bam_options(comment_separation_char: u8) -> BamSinkOptions {
    BamSinkOptions {
        comment_separation_char,
        tag_to_bam_tags: Vec::new(),
        tag_to_reference: None,
        header: SharedBamHeader::default(),
    }
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
    let mut references_from_bam: Option<PathBuf> = None;

    if let Some(sep) = partial.comment_separation_char.as_ref() {
        comment_separation_char = *sep;
    }

    if let Some(map_and_keys) = partial.tag_to_bam_tag.value.as_ref() {
        for (tag_label, bam_tag) in &map_and_keys.map {
            if let Some(bam_tag) = bam_tag.as_ref() {
                tag_to_bam_tags.push((bam_tag.0, tag_label.to_string()));
            }
        }
    }

    if let Some(tag_to_ref) = partial
        .tag_to_reference
        .value
        .as_mut()
        .and_then(|x| x.as_mut())
    {
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
                tag_to_ref.references_from_barcodes.state = TomlValueState::new_validation_failed(
                    "Barcode section not found for output.bam.tag_to_reference",
                );
            }
        } else if let Some(path) = tag_to_ref
            .references_from_bam
            .as_ref()
            .and_then(|o| o.as_ref())
            .cloned()
        {
            // Carry the path; the file is read lazily when the BAM writer is
            // opened (see `BamSinkOptions::resolve_header`). Deferring the IO
            // keeps `validate` filesystem-free.
            references_from_bam = Some(PathBuf::from(path));
        }
    }

    // For the `from_bam` case the references are read (and the header built) on
    // first writer creation; the barcodes case already has the references in
    // hand. Either way the header is resolved once and shared across all sinks.
    let header = match references_from_bam {
        Some(path) => SharedBamHeader::from_bam(path),
        None => SharedBamHeader::from_reference_sequences(Arc::new(reference_sequences)),
    };
    let opts = BamSinkOptions {
        comment_separation_char,
        tag_to_bam_tags,
        tag_to_reference,
        header,
    };
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

    if !matches!(partial.merge_demultiplexed.as_ref(), Some(Some(true))) {
        return;
    }

    if output_empty && interleave_empty {
        partial.merge_demultiplexed.state = TomlValueState::Custom {
            spans: vec![
                (
                    partial.merge_demultiplexed.span(),
                    "Incompatible with empty outputs".to_string(),
                ),
                (output_span, "These output segments are empty".to_string()),
            ],
        };
        partial.merge_demultiplexed.help = Some(
            "Either remove 'merge_demultiplexed' or specify either output segments or interleaved output.".to_string(),
        );
    }

    match partial.tag_to_reference.as_ref() {
        Some(Some(tag_to_reference)) => {
            if let Some(ref_label) = tag_to_reference.tag.as_ref()
                && !available_demultiplex_labels.contains(ref_label)
            {
                partial.merge_demultiplexed.state = TomlValueState::new_validation_failed(format!(
                    "No Demultiplex step found that had in_label = {ref_label}",
                ));
                if available_demultiplex_labels.is_empty() {
                    partial.merge_demultiplexed.help = Some(
                        "No suitable Demultiplex step found. Make sure you have a Demultiplex step with lookup_mode = 'lookup' and an in_label that matches output.bam.tag_to_reference.tag.".to_string(),
                    );
                } else {
                    partial.merge_demultiplexed.help = Some(format!(
                        "Either add a Demultiplex step or reuse one of the following: {}",
                        offer_alternatives("", available_demultiplex_labels)
                    ));
                }
            }
        }
        Some(None) => {
            partial.merge_demultiplexed.state = TomlValueState::new_validation_failed(
                "merge_demultiplexed requires tag_to_reference to be set.",
            );
            partial.merge_demultiplexed.help = Some(
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

        // Each fastqrab tag exported to a BAM auxiliary tag must exist. The
        // map keys carry the raw label text (`TomlValue<String>`); the
        // parsed `TagLabel`s are the keys of `map`.
        if let Some(map_and_keys) = inner.tag_to_bam_tag.value.as_mut() {
            for (tv_key, tag_label) in map_and_keys.keys.iter_mut().zip(map_and_keys.map.keys()) {
                if tv_key.is_ok() {
                    used_tags.push(Some(UsedTag {
                        name: tag_label.clone(),
                        accepted_tag_types: ANY_TAG_TYPE,
                        toml_source: Rc::new(RefCell::new((&mut tv_key.state, &mut tv_key.help))),
                        further_help: None,
                    }));
                }
            }
        }
        // The reference-selecting tag must exist; `from_barcodes` names a
        // barcode section we then count as "used".
        if let Some(tag_to_ref) = inner
            .tag_to_reference
            .value
            .as_mut()
            .and_then(|x| x.as_mut())
        {
            if let Some(section) = tag_to_ref
                .references_from_barcodes
                .as_ref()
                .and_then(|o| o.as_ref())
            {
                used_barcodes.insert(TagLabel::Normal(section.clone()));
            }
            if let Some(tag_name) = tag_to_ref.tag.as_ref() {
                let name = TagLabel::Normal(tag_name.clone());
                if !used_tags
                    .iter()
                    .any(|other| other.as_ref().map(|x| x.name == name).unwrap_or(false))
                {
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

    fn declare_output_files(&self) -> Option<Vec<OutputDeclaration>> {
        if let Some(inner) = self.toml_value.as_ref() {
            let comment_separation_char = inner
                .comment_separation_char
                .as_ref()
                .copied()
                .unwrap_or(b' ');

            // Reference sequences / header / tag exports are resolved earlier by
            // `resolve_output_bam`; fall back to a minimal header if (somehow) absent.
            let bam_options = inner
                .resolved_bam_options
                .clone()
                .flatten()
                .unwrap_or_else(|| declare_bam_options(comment_separation_char));

            let segments = collect_segment_list(&inner.output);
            let interleave = interleave_present(&inner.interleave)
                .then(|| collect_segment_list(&inner.interleave));
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
                    false,
                    *inner.output_hash_compressed.unwrap_ref(),
                ),
                chunksize: inner.chunksize.as_ref().and_then(|x| x.as_ref()).copied(),
                bam_options: Some(bam_options),
                span: self.toml_value.span(),
            };
            Some(build_record_declarations(&spec))
        } else {
            Some(vec![]) //there should be output files, but we can't name them.
        }
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
        _input_files: &mut StepInputFiles,
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
