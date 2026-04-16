
use bstr::BString;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use hamming_resonate::HammingResonator;

use crate::transformations::prelude::*;
use fastqrab_io::io::apply_to_read_names_and_sequences;

/// Assign each query sequence to the closest entry in a named reference database.
///
/// The reference is a FASTA or FASTQ file (optionally gzip-compressed).  Every
/// sequence in the file must have the same length as the query.  The step
/// builds a Hamming-distance index at initialisation and stores the matched
/// reference name as a string tag.  When no entry is within `max_hamming_distance`
/// the tag is set to `Missing`, which lets a downstream `FilterByTag` discard
/// unassigned reads.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct AssignToReference {
    /// Tag containing the query sequence (must be a String or Location tag
    /// whose sequences have the same length as the reference entries).
    pub in_label: TagLabel,

    /// Output tag where the matched reference name is stored.
    pub out_label: TagLabel,

    /// Path to the reference file (FASTA or FASTQ, plain or gzip-compressed).
    #[tpd(alias = "filename")]
    pub reference: String,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub reference_read_comment_character: Option<u8>,

    /// Maximum Hamming distance allowed for a match.  Use 0 for exact matches
    /// only.
    pub max_hamming_distance: u32,

    // ── built during init ───────────────────────────────────────────────────
    #[tpd(skip, default)]
    #[schemars(skip)]
    pub seq_to_name: IndexMap<BString, String>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub resonator: Option<Arc<HammingResonator>>,
}

impl VerifyIn<PartialConfig> for PartialAssignToReference {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(out_label) = self.out_label.as_ref()
            && let Some(in_label) = self.in_label.as_ref()
            && out_label == in_label
        {
            let spans = vec![
                (self.in_label.span(), "The same as out_label".to_string()),
                (self.out_label.span(), "The same as in_label".to_string()),
            ];
            self.out_label.state = TomlValueState::Custom { spans };
            self.out_label.help =
                Some("Use different tag names for in_label and out_label.".to_string());
        }

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialAssignToReference> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::String),
                used_tags: vec![
                    inner
                        .in_label
                        .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                ],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for AssignToReference {
    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut entries: Vec<(BString, String)> = Vec::new();

        apply_to_read_names_and_sequences(
            &self.reference,
            &mut |name: &[u8], seq: &[u8]| {
                let name_str = if let Some(comment_char) = self.reference_read_comment_character
                    && let Some(pos) = name.iter().position(|&c| c == comment_char)
                {
                    let name_without_comment = &name[..pos];
                    String::from_utf8_lossy(name_without_comment).into_owned()
                } else {
                    String::from_utf8_lossy(name).into_owned()
                };

                entries.push((BString::from(seq), name_str));
            },
            input_info.use_rapidgzip,
        )?;

        if entries.is_empty() {
            bail!(
                "AssignToReference: reference file '{}' contains no sequences.",
                self.reference
            );
        }

        let seqs: Vec<BString> = entries.iter().map(|(s, _)| s.clone()).collect();
        let seq_to_name: IndexMap<BString, String> = entries.into_iter().collect();

        if seqs.len() != seq_to_name.len() {
            let mut counts = IndexMap::new();
            for seq in seqs {
                *counts.entry(seq).or_insert(0) += 1;
            }
            let mut duplicates: Vec<_> =
                counts.into_iter().filter(|(_, count)| *count > 1).collect();
            let had_more = if duplicates.len() > 10 {
                duplicates.truncate(10);
                true
            } else {
                false
            };
            let mut duplicate_string = duplicates
                .into_iter()
                .map(|(seq, count)| format!("{} ({} times)", String::from_utf8_lossy(&seq), count))
                .collect::<Vec<_>>()
                .join(", ");
            if had_more {
                duplicate_string.push_str(", ...");
            }
            bail!(
                "AssignToReference: reference file '{}' contains duplicate sequences.\n\
                This is not supported, check your input.\n\
                Duplicated sequences: {duplicate_string}",
                self.reference
            );
        }

        let resonator = HammingResonator::new(seqs, self.max_hamming_distance)
            .map_err(|e| anyhow::anyhow!("Failed to build reference index: {e}"))?;

        for seq in seq_to_name.keys() {
            let hit = resonator.query(seq.as_ref()).map_err(|e| {
                anyhow::anyhow!("Failed to query reference index during validation: {e}")
            })?;
            match hit.len() {
                0 => {
                    panic!(
                        "HammingResonator did not return a hit for a sequence that was indexed. This should not happen, check the implementation of HammingResonator."
                    );
                }
                1 => {}
                _ => {
                let mut hit = hit;
                hit.sort();
                    bail!(
                        "The reference sequence {seq} had more than one hit within the specificied max hamming distance {}\n\
                        Hits: {hit:?}",
                        self.max_hamming_distance
                    );
                }
            }
        }

        self.seq_to_name = seq_to_name;
        self.resonator = Some(Arc::new(resonator));
        Ok(None)
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let resonator = self
            .resonator
            .as_ref()
            .expect("resonator must be set during init");
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let mut output_tags: Vec<TagValue> = Vec::with_capacity(input_tags.len());

        for input_tag in input_tags {
            let query_seq: Option<&[u8]> = match input_tag {
                TagValue::String(s) => Some(s.as_ref()),
                TagValue::Location(hits) => hits.0.first().map(|hit| hit.sequence.as_ref()),
                TagValue::Missing => None,
                TagValue::Bool(_) | TagValue::Numeric(_) => None,
            };

            let tag_value = if let Some(query) = query_seq {
                let hits = resonator
                    .query(query.into())
                    .map_err(|e| anyhow::anyhow!("AssignToReference query failed: {e}"))?;
                if let Some((matched_seq, _dist)) = hits.first() {
                    match self.seq_to_name.get(*matched_seq) {
                        Some(name) => TagValue::String(BString::from(name.as_bytes())),
                        None => TagValue::Missing, // cov:excl-line
                    }
                } else {
                    TagValue::Missing
                }
            } else {
                TagValue::Missing
            };

            output_tags.push(tag_value);
        }

        block.tags.insert(self.out_label.clone(), output_tags);
        Ok((block, true))
    }
}
