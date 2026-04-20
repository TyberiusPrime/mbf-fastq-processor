use hamming_resonate::HammingResonator;
use indexmap::IndexMap;

use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_dna::dna::init_hamming_resonator;

/// Correct a tag (extracted region) to known barcodes

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct HammingCorrect {
    /// Input tag to correct
    pub in_label: TagLabel,
    /// Output tag to store corrected result
    pub out_label: TagLabel,
    /// Reference to barcodes section
    pub barcodes: TagLabel,
    /// Maximum hamming distance for correction
    pub max_hamming_distance: u8,
    /// What to do when no match is found
    pub on_no_match: OnNoMatch,

    /// names are considered identical if they match up to the first name_split_character
    /// for must-have-hamming-distance considerations
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "name_split_char")]
    pub name_split_character: Option<u8>,

    #[tpd(skip)]
    #[schemars(skip)]
    seq_to_name: Arc<IndexMap<BString, String>>,

    #[tpd(skip)]
    #[schemars(skip)]
    resonator: Arc<HammingResonator>,
}

impl VerifyIn<PartialConfig> for PartialHammingCorrect {
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
                (self.in_label.span(), "The same as outlabel".to_string()),
                (self.out_label.span(), "The same as inlabel".to_string()),
            ];
            self.out_label.state = TomlValueState::Custom { spans };
            self.out_label.help =
                Some("Please use different tag names for the input and output labels to avoid overwriting the source tag.".to_string())
                ;
        }
        self.max_hamming_distance.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "Must be greater than 0 to perform correction",
                    Some("Leave off the HammingCorrect step if no correction is desired."),
                ))
            } else {
                Ok(())
            }
        });

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
                        self.resonator = Some(Arc::new(
                            init_hamming_resonator(seq_to_name, *max_hamming_distance, None)
                                .map_err(|e| {
                                    ValidationFailure::new(
                                        format!("Failure to initialize"),
                                        Some(format!(
                                            "Error: {e}\n\
                                    Verify your barcodes, they must be of the same length \
                                        and disjoint under your max_hamming_distance \
                                        for a given reference target."
                                        )),
                                    )
                                })?,
                        ));
                        self.seq_to_name = Some(seq_to_name.clone());
                    }
                    // otherwise the barcode section wasn't ok and we'll never
                    // be turned into a concrete HammingCorrect.
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
            return Err(ValidationFailure::new(
                "HammingCorrect step requires a barcodes section to be defined in the config.",
                Some(&format!("See {}", crate::link_docs("barcodes"))),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, JsonSchema)]
#[tpd]
pub enum OnNoMatch {
    Remove,
    Empty,
    Keep,
}

impl TagUser for PartialTaggedVariant<PartialHammingCorrect> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Location),
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

impl HammingCorrect {
    fn match_sequence(&self, sequence: &BStr) -> Result<Option<&BStr>> {
        let matched = self
            .resonator
            .query(&BStr::new(&sequence))
            .map_err(|e| anyhow::anyhow!("HammingCorrect query failed: {e}"))?;
        if matched.is_empty() {
            return Ok(None);
        } else if matched.len() == 1 {
            return Ok(Some(matched[0].0));
        } else {
            let matched_plus_seq: Vec<_> = matched
                .iter()
                .map(|(seq, dist)| {
                    let seq_name: BString = self
                        .seq_to_name
                        .get(*seq)
                        .expect("Must be in there?!")
                        .as_bytes()
                        .into();
                    let seq_name = match self.name_split_character {
                        Some(split_char) => seq_name
                            .splitn(2, |&c| c == split_char)
                            .next()
                            .unwrap_or(&seq_name)
                            .into(),
                        None => seq_name,
                    };
                    (seq, dist, seq_name)
                })
                .collect();
            let all_same_reference = matched_plus_seq
                .iter()
                .all(|(_seq, _dist, name)| *name == matched_plus_seq[0].2);
            if all_same_reference {
                // If all matched sequences correspond to the same reference, we can still correct,
                // but we won't know which sequence is the best match,
                let mut matched = matched;
                matched.sort_by_key(|(seq, dist)| (*dist, *seq));
                return Ok(Some(matched[0].0));
            } else {
                bail!(
                    "HammingCorrect on in_label={} \n\
                     Uncorrectable sequence '{}', \n\
                     matches multiple sequences within hamming distance: {:?}.\n
                    Maybe set name_split_character if you want to consider these as the same reference?",
                    self.in_label,
                    BStr::new(&sequence),
                    matched_plus_seq
                );
            }
        }
    }
}

impl Step for HammingCorrect {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let mut output_hits = Vec::new();

        for input_tag in input_tags {
            match input_tag {
                TagValue::Location(hit_sequences) => {
                    let seq = hit_sequences.joined_sequence(None);
                    match self.match_sequence(BStr::new(&seq))? {
                        Some(matched_seq) => {
                            let new_hit = Hit {
                                sequence: matched_seq.into(),
                                location: if hit_sequences.0.len() == 1 {
                                    hit_sequences.0[0].location.clone()
                                } else {
                                    None
                                },
                            };
                            output_hits.push(TagValue::Location(Hits::new_multiple(vec![new_hit])));
                        }
                        None => {
                            match self.on_no_match {
                                OnNoMatch::Remove => {
                                    // Create empty tag value
                                    output_hits.push(TagValue::Missing);
                                }
                                OnNoMatch::Empty => {
                                    // Create hit with empty sequence
                                    output_hits.push(TagValue::Location(Hits(vec![])));
                                }
                                OnNoMatch::Keep => {
                                    // Keep original hit unchanged
                                    output_hits.push(input_tag.clone());
                                }
                            }
                        }
                    }
                }
                TagValue::String(hit_string) => {
                    match self.match_sequence(hit_string.as_ref())? {
                        Some(matched_seq) => output_hits.push(TagValue::String(matched_seq.into())),
                        None => {
                            match self.on_no_match {
                                OnNoMatch::Remove => {
                                    // Create empty tag value
                                    output_hits.push(TagValue::Missing);
                                }
                                OnNoMatch::Empty => {
                                    // Create hit with empty sequence
                                    output_hits.push(TagValue::String("".into()));
                                }
                                OnNoMatch::Keep => {
                                    // Keep original hit unchanged
                                    output_hits.push(input_tag.clone());
                                }
                            }
                        }
                    }
                }
                TagValue::Missing => {
                    output_hits.push(TagValue::Missing);
                }
                // cov:excl-start
                TagValue::Bool(_) | TagValue::Numeric(_) => {
                    unreachable!(); // we verify that it's a location tag in validation
                } // cov:excl-stop
            }
        }

        // Add the corrected tags to the output
        block.tags.insert(self.out_label.clone(), output_hits);

        Ok((block, true))
    }
}
