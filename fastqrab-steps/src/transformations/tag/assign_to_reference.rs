use bstr::BString;
use fastqrab_dna::dna::init_hamming_resonator;
use hamming_resonate::HammingResonator;

use crate::transformations::prelude::*;

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
    ///
    /// Reference to barcodes section
    pub barcodes: TagLabel,

    /// Maximum Hamming distance allowed for a match.  Use 0 for exact matches
    /// only.
    pub max_hamming_distance: u8,

    // ── built during init ───────────────────────────────────────────────────
    #[tpd(skip)]
    #[schemars(skip)]
    seq_to_name: Arc<IndexMap<BString, String>>,

    #[tpd(skip)]
    #[schemars(skip)]
    resonator: Arc<HammingResonator>,
}

impl VerifyIn<PartialConfig> for PartialAssignToReference {
    fn verify(
        &mut self,
        parent: &PartialConfig,
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

        if let Some(barcodes_to_use) = self.barcodes.as_ref()
            && let Some(barcode_data) = parent.barcodes.as_ref()
            && let Some(barcodes_data) = barcode_data
        {
            match barcodes_data.map.get(barcodes_to_use) {
                Some(barcodes_section) => {
                    if let Some(barcodes_section) = barcodes_section.as_ref()
                        && let Some(seq_to_name) = &barcodes_section.seq_to_name
                        && let Some(max_hamming_distance) = self.max_hamming_distance.as_ref()
                    {
                        self.resonator = Some(Arc::new(init_hamming_resonator(
                            seq_to_name, *max_hamming_distance)
                            .map_err(|e| {
                                ValidationFailure::new(
                                    format!("Failure to initialize"),
                                    Some(format!(
                                        "Error: {e}\n\
                                    Verify your barcodes, they must be of the same length and disjoint under your max_hamming_distance."
                                    )),
                                )
                            })?));
                        self.seq_to_name = Some(seq_to_name.clone());
                    }
                    // otherwise the barcode section wasn't ok and we'll never
                    // be turned into a concrete AssignToReference.
                }
                None => {
                    self.barcodes.help = Some(offer_alternatives(
                        barcodes_to_use.as_ref(),
                        &barcodes_data.map.keys().collect::<Vec<_>>(),
                    ));

                    self.barcodes.state = TomlValueState::ValidationFailed {
                        message: "Barcodes section not found".to_string(),
                    };
                    return Ok(());
                }
            }
        } else {
            // return Err(ValidationFailure::new(
            //     "AssignToReference step requires a barcodes section to be defined in the config.",
            //     Some(&format!("See {}", crate::link_docs("barcodes"))),
            // ));
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
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let resonator = &self.resonator;
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
