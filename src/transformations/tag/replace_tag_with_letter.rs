#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    config::{deser::u8_from_char_or_number, Target},
    Demultiplexed,
};

use super::super::Step;
use super::common::default_replacement_letter;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReplaceTagWithLetter {
    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    #[serde(default = "default_replacement_letter")]
    pub letter: u8,
}

impl Step for ReplaceTagWithLetter {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.apply_mut_with_tag(&self.label, |read1, read2, index1, index2, tag_val| {
            if let Some(hit) = tag_val.as_sequence() {
                for region in &hit.0 {
                    if let Some(location) = &region.location {
                        let read: &mut crate::io::WrappedFastQReadMut = match location.target {
                            Target::Read1 => read1,
                            Target::Read2 => read2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index1 => index1
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index2 => index2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                        };

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
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }

        (block, true)
    }
}