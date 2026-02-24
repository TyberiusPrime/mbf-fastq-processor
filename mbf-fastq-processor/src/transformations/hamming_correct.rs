#![allow(clippy::unnecessary_wraps)]
use indexmap::IndexMap;
use toml_pretty_deser::suggest_alternatives;

use crate::{
    dna::{hamming, iupac_hamming_distance},
    transformations::prelude::*,
};

use crate::dna::{Hits, TagValue};

/// Correct a tag (extracted region) to known barcodes

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct HammingCorrect {
    /// Input tag to correct
    pub in_label: String,
    /// Output tag to store corrected result
    pub out_label: String,
    /// Reference to barcodes section
    pub barcodes: String,
    /// Maximum hamming distance for correction
    pub max_hamming_distance: u8,
    /// What to do when no match is found
    pub on_no_match: OnNoMatch,

    #[tpd(skip)]
    #[schemars(skip)]
    pub resolved_barcodes: IndexMap<BString, String>,
    #[tpd(skip)]
    #[schemars(skip)]
    pub had_iupac: bool,
}

impl VerifyIn<PartialConfig> for PartialHammingCorrect {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.in_label.verify(|v| {
            if v.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
        self.out_label.verify(|v| {
            if v.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
        self.barcodes.verify(|v| {
            if v.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
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
                    "Must be greater than 0 to perform correction. Leave off the HammingCorrect step if no correction is desired.",
                    None,
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
                    let barcodes_section: IndexMap<BString, String> = barcodes_section
                        .as_ref()
                        .expect("parent ok")
                        .barcode_to_name
                        .as_ref()
                        .expect("parent ok2")
                        .map
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                v.value.as_ref().expect("parent was ok").to_owned(),
                            )
                        })
                        .collect();
                    // Copy the resolved barcodes

                    // Check if any barcode contains IUPAC ambiguous bases
                    self.had_iupac = Some(
                        barcodes_section
                            .keys()
                            .any(|x| crate::dna::contains_iupac_ambigous(x)),
                    );
                    self.resolved_barcodes = Some(barcodes_section);
                }
                None => {
                    self.barcodes.help = Some(suggest_alternatives(
                        barcodes_to_use,
                        &barcodes_data.map.keys().collect::<Vec<_>>(),
                    ));

                    self.barcodes.state = TomlValueState::ValidationFailed {
                        message: "Barcodes section not found".to_string(),
                    };
                }
            }
            assert!(
                self.resolved_barcodes.is_some(),
                "Barcodes not resolved. Bug"
            );
        } else {
            return Err(ValidationFailure::new(
                "HammingCorrect step requires a barcodes section to be defined in the config.",
                None, //todo link
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

impl Step for HammingCorrect {
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.out_label.clone(), TagValueType::Location))
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(
            self.in_label.clone(),
            &[TagValueType::Location, TagValueType::String],
        )])
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let barcodes = &self.resolved_barcodes;
        let mut output_hits = Vec::new();

        for input_tag in input_tags {
            match input_tag {
                TagValue::Location(hit_sequences) => {
                    let corrected_hits = correct_barcodes(
                        barcodes,
                        hit_sequences.0.iter().map(|hit| (hit, &hit.sequence)),
                        self.on_no_match,
                        self.max_hamming_distance,
                        self.had_iupac,
                    );
                    if corrected_hits.is_empty() {
                        match self.on_no_match {
                            OnNoMatch::Remove => {
                                // Create empty tag value
                                output_hits.push(TagValue::Missing);
                            }
                            _ => {
                                // This shouldn't happen as we handle it above
                                //output_hits.push(TagValue::Sequence(Hits(corrected_hits)));
                                unreachable!();
                            }
                        }
                    } else {
                        output_hits.push(TagValue::Location(Hits(corrected_hits)));
                    }
                }
                TagValue::String(hit_string) => {
                    let mut corrected_hits: Vec<BString> = correct_barcodes(
                        barcodes,
                        [hit_string].into_iter().map(|hit| (hit, hit)),
                        self.on_no_match,
                        self.max_hamming_distance,
                        self.had_iupac,
                    );
                    if corrected_hits.is_empty() {
                        match self.on_no_match {
                            OnNoMatch::Remove => {
                                // Create empty tag value
                                output_hits.push(TagValue::Missing);
                            }
                            _ => {
                                // This shouldn't happen as we handle it above
                                unreachable!();
                            }
                        }
                    } else {
                        output_hits.push(TagValue::String(
                            corrected_hits
                                .pop()
                                .expect("corrected_hits must have at least one element"),
                        ));
                    }
                }
                TagValue::Missing => {
                    output_hits.push(TagValue::Missing);
                }
                TagValue::Bool(_) | TagValue::Numeric(_) => {
                    unreachable!(); // we verify that it's a location tag in validation
                }
            }
        }

        // Add the corrected tags to the output
        block.tags.insert(self.out_label.clone(), output_hits);

        Ok((block, true))
    }
}

trait WithUpdatedSequence {
    fn clone_with_sequence(&self, sequence: &BString) -> Self;
}

impl WithUpdatedSequence for crate::dna::Hit {
    fn clone_with_sequence(&self, sequence: &BString) -> Self {
        let mut new_hit = self.clone();
        new_hit.sequence = sequence.clone();
        new_hit
    }
}

impl WithUpdatedSequence for BString {
    fn clone_with_sequence(&self, sequence: &BString) -> Self {
        sequence.clone()
    }
}

fn correct_barcodes<'a, T: Clone + WithUpdatedSequence + 'a>(
    barcodes: &IndexMap<BString, String>,
    hit_sequences: impl Iterator<Item = (&'a T, &'a BString)>,
    on_no_match: OnNoMatch,
    max_hamming_distance: u8,
    had_iupac: bool,
) -> Vec<T> {
    let mut corrected_hits = Vec::new();
    for (hit_seq, sequence) in hit_sequences {
        let mut found_match = false;

        // Try exact match first
        if barcodes.contains_key(sequence) {
            corrected_hits.push(hit_seq.clone());
            found_match = true;
        } else if max_hamming_distance > 0 {
            // Try hamming distance correction
            // Use IUPAC hamming distance
            for barcode in barcodes.keys() {
                let distance = if had_iupac {
                    iupac_hamming_distance(barcode, sequence)
                } else {
                    hamming(barcode, sequence)
                        .try_into()
                        .expect("hamming distance conversion should succeed")
                };
                if distance.try_into().unwrap_or(255u8) <= max_hamming_distance {
                    // Create corrected hit with new sequence
                    corrected_hits.push(hit_seq.clone_with_sequence(barcode));
                    found_match = true;
                    break;
                }
            }
        }

        if !found_match {
            match on_no_match {
                OnNoMatch::Remove => {
                    // Don't add to corrected_hits - effectively removes the tag
                }
                OnNoMatch::Empty => {
                    // Create hit with empty sequence
                    corrected_hits.push(hit_seq.clone_with_sequence(&BString::new(vec![])));
                }
                OnNoMatch::Keep => {
                    // Keep original hit unchanged
                    corrected_hits.push(hit_seq.clone());
                }
            }
        }
    }
    corrected_hits
}
