//eserde false positives
#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::{tpd_extract_u8_from_byte_or_char, u8_from_char_or_number};

/// Replace all bases with this (region) tag with one base

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReplaceTagWithLetter {
    pub in_label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    /// Provide the replacement letter as a single character (e.g., 'N') or its ASCII numeric value (e.g., 78 for 'N').
    pub letter: u8,
}

impl VerifyFromToml for PartialReplaceTagWithLetter {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.letter = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_letter(helper, false, false),
            self.tpd_get_letter(helper, false, false),
            true,
            helper,
        );
        self
    }
}

impl Step for ReplaceTagWithLetter {
    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.apply_mut_with_tag(&self.in_label, |reads, tag_val| {
            if let Some(hit) = tag_val.as_sequence() {
                for region in &hit.0 {
                    if let Some(location) = &region.location {
                        let read = &mut reads[location.segment_index.get_index()];

                        // Replace the sequence bases in the specified region with the replacement letter
                        let seq = read.seq_mut();
                        for i in location.start..location.start + location.len {
                            if i < seq.len() {
                                seq[i] = self.letter;
                            }
                        }
                    }
                }
            }
        });

        Ok((block, true))
    }
}
