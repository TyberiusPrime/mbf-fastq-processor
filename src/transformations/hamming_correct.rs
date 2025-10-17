#![allow(clippy::unnecessary_wraps)]
use anyhow::{Result, bail};
use bstr::BString;
use std::collections::BTreeMap;
use std::path::Path;

use super::{InputInfo, Step, TagValueType};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use crate::dna::{Hits, TagValue};
use crate::io::FastQBlocksCombined;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct HammingCorrect {
    /// Input tag to correct
    pub label_in: String,
    /// Output tag to store corrected result
    pub label_out: String,
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

#[derive(eserde::Deserialize, Debug, Validate, Clone, Copy)]
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
        Some((self.label_out.clone(), TagValueType::Location))
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.label_in == self.label_out {
            bail!("label_in and label_out cannot be the same");
        }
        Ok(())
    }


    fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
        Some(vec![(self.label_in.clone(), TagValueType::Location)])
    }

    fn resolve_config_references(
        &mut self,
        barcodes_data: &std::collections::HashMap<String, crate::config::Barcodes>,
    ) -> Result<()> {
        // Resolve the barcodes reference
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
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        if self.resolved_barcodes.is_none() {
            bail!("Barcodes not resolved. This should have been done during config resolution.");
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<(FastQBlocksCombined, bool)> {
        let input_hits = block
            .tags
            .as_ref()
            .expect("No tags available")
            .get(&self.label_in)
            .expect("Input tag not found");

        let barcodes = self.resolved_barcodes.as_ref().unwrap();
        let mut output_hits = Vec::new();

        for input_hit in input_hits {
            match input_hit {
                TagValue::Sequence(hit_sequences) => {
                    let mut corrected_hits = Vec::new();

                    for hit_seq in &hit_sequences.0 {
                        let sequence = &hit_seq.sequence;
                        let mut found_match = false;

                        // Try exact match first
                        if barcodes.contains_key(sequence) {
                            corrected_hits.push(hit_seq.clone());
                            found_match = true;
                        } else if self.max_hamming_distance > 0 {
                            // Try hamming distance correction
                            // Use IUPAC hamming distance
                            for barcode in barcodes.keys() {
                                let distance = if self.had_iupac {
                                    crate::dna::iupac_hamming_distance(barcode, sequence)
                                } else {
                                    bio::alignment::distance::hamming(barcode, sequence)
                                        .try_into()
                                        .unwrap()
                                };
                                if distance.try_into().unwrap_or(255u8) <= self.max_hamming_distance
                                {
                                    // Create corrected hit with new sequence
                                    let mut corrected_hit = hit_seq.clone();
                                    corrected_hit.sequence = barcode.clone();
                                    corrected_hits.push(corrected_hit);
                                    found_match = true;
                                    break;
                                }
                            }
                        }

                        if !found_match {
                            match self.on_no_match {
                                OnNoMatch::Remove => {
                                    // Don't add to corrected_hits - effectively removes the tag
                                }
                                OnNoMatch::Empty => {
                                    // Create hit with empty sequence
                                    let mut empty_hit = hit_seq.clone();
                                    empty_hit.sequence = BString::new(Vec::new());
                                    corrected_hits.push(empty_hit);
                                }
                                OnNoMatch::Keep => {
                                    // Keep original hit unchanged
                                    corrected_hits.push(hit_seq.clone());
                                }
                            }
                        }
                    }

                    if corrected_hits.is_empty() {
                        match self.on_no_match {
                            OnNoMatch::Remove => {
                                // Create empty tag value
                                output_hits.push(TagValue::Missing);
                            }
                            _ => {
                                // This shouldn't happen as we handle it above
                                output_hits.push(TagValue::Sequence(Hits(corrected_hits)));
                            }
                        }
                    } else {
                        output_hits.push(TagValue::Sequence(Hits(corrected_hits)));
                    }
                }
                TagValue::Bool(_) | TagValue::Numeric(_) | TagValue::Missing => {
                    unreachable!(); // we verify that it's a location tag in validation
                }
            }
        }

        // Add the corrected tags to the output
        if block.tags.is_none() {
            block.tags = Some(std::collections::HashMap::new());
        }
        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label_out.clone(), output_hits);

        Ok((block, true))
    }
}
