use bstr::BString;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
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

    /// names are considered identical if they match up to the first name_split_character
    /// for must-have-hamming-distance considerations
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char", alias = "name_split_char")]
    pub name_split_character: Option<u8>,

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
                            seq_to_name,
                            *max_hamming_distance,
                        )?));
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
                match hits.len() {
                    0 => TagValue::Missing,
                    1 => {
                        let matched_seq = &hits[0].0; //safe we just checked 
                        match self.seq_to_name.get(*matched_seq) {
                            Some(name) => TagValue::String(BString::from(name.as_bytes())),
                            None => TagValue::Missing, // cov:excl-line
                        }
                    }
                    _ => {
                        let matched_plus_seq: Vec<_> = hits
                            .iter()
                            .map(|(seq, dist)| {
                                let org_seq_name: BString = self
                                    .seq_to_name
                                    .get(*seq)
                                    .expect("Must be in there?!")
                                    .as_bytes()
                                    .into();
                                let seq_name = match self.name_split_character {
                                    Some(split_char) => org_seq_name
                                        .splitn(2, |&c| c == split_char)
                                        .next()
                                        .unwrap_or(&org_seq_name)
                                        .into(),
                                    None => org_seq_name.clone(),
                                };
                                (seq, dist, seq_name, org_seq_name)
                            })
                            .collect();
                        let all_same_reference =
                            matched_plus_seq
                                .iter()
                                .all(|(_seq, _dist, split_name, _org_name)| {
                                    *split_name == matched_plus_seq[0].2
                                });
                        if all_same_reference {
                            TagValue::String(matched_plus_seq[0].3.clone())
                        } else {
                            bail!(
                                "AssignToReference on in_label={} \n\
                             Uncorrectable sequence '{}', \n\
                             matches multiple sequences within hamming distance: {:?}\n\
                            Maybe set/check name_split_character if you want to consider these as the same reference?",
                                self.in_label,
                                BStr::new(&query),
                                matched_plus_seq
                            );
                        }
                    }
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
