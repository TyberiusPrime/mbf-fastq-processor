#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

use bstr::BString;
use std::collections::BTreeMap;
use std::path::Path;

use crate::dna::{Hits, TagValue};
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
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

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub resolved_barcodes: Option<BTreeMap<BString, String>>,
    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub had_iupac: bool,
}

#[derive(eserde::Deserialize, Debug, Validate, Clone, Copy, JsonSchema)]
pub enum OnNoMatch {
    #[serde(alias = "remove")]
    Remove,
    #[serde(alias = "empty")]
    Empty,
    #[serde(alias = "keep")]
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
        if self.in_label == self.out_label {
            bail!(
                "HammingCorrect: 'in_label' and 'out_label' cannot be the same. Please use different tag names for the input and output labels to avoid overwriting the source tag."
            );
        }
        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(
            self.in_label.clone(),
            &[TagValueType::Location, TagValueType::String],
        )])
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let barcodes_data = &input_info.barcodes_data;
        match barcodes_data.get(&self.barcodes) {
            Some(barcodes_section) => {
                // Copy the resolved barcodes
                self.resolved_barcodes = Some(barcodes_section.barcode_to_name.clone());

                // Check if any barcode contains IUPAC ambiguous bases
                self.had_iupac = barcodes_section
                    .barcode_to_name
                    .keys()
                    .any(|x| crate::dna::contains_iupac_ambigous(x));
            }
            None => {
                bail!(
                    "Barcodes section '{}' not found. Available sections: {:?}",
                    self.barcodes,
                    barcodes_data.keys().collect::<Vec<_>>()
                );
            }
        }
        assert!(
            self.resolved_barcodes.is_some(),
            "Barcodes not resolved. Bug"
        );

        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_tags = block.tags.get(&self.in_label).expect("Input tag not found");

        let barcodes = self.resolved_barcodes.as_ref().expect("resolved_barcodes must be set during initialization");
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
                        output_hits.push(TagValue::String(corrected_hits.pop().expect("corrected_hits must have at least one element")));
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
    barcodes: &BTreeMap<BString, String>,
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
                    crate::dna::iupac_hamming_distance(barcode, sequence)
                } else {
                    bio::alignment::distance::hamming(barcode, sequence)
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
