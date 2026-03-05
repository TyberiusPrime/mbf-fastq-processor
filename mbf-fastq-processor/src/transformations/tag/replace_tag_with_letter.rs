//eserde false positives
#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::tpd_adapt_u8_from_byte_or_char;

/// Replace all bases with this (region) tag with one base

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReplaceTagWithLetter {
    pub in_label: TagLabel,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    /// Provide the replacement letter as a single character (e.g., 'N') or its ASCII numeric value (e.g., 78 for 'N').
    pub letter: u8,
}

impl VerifyIn<PartialConfig> for PartialReplaceTagWithLetter {}

impl TagUser for PartialTaggedVariant<PartialReplaceTagWithLetter> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        TagUsageInfo {
            used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location][..])],
            ..Default::default()
        }
    }
}

impl Step for ReplaceTagWithLetter {
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