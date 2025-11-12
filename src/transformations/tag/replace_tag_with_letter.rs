//eserde false positives
#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::u8_from_char_or_number;

use super::super::tag::default_replacement_letter;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplaceTagWithLetter {
    pub in_label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    #[serde(default = "default_replacement_letter")]
    pub letter: u8,
}

impl Step for ReplaceTagWithLetter {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    fn apply(
        &mut self,
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

        // Remove the consumed tag after processing

        block.tags.remove(&self.in_label);

        Ok((block, true))
    }
}
