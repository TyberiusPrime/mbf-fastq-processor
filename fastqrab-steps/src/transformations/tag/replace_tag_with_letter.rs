use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;

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
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location])],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ReplaceTagWithLetter {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.apply_mut_with_location_tag(&self.in_label, |seq, hit| {
            for &(start, len) in hit.iter() {
                // Replace the sequence bases in the specified region with the replacement letter
                //robust against out-of-bounds, just in case
                for i in start..(start + len).min(seq.len().try_into().expect("exceeds u32")) {
                    seq[i as usize] = self.letter;
                }
            }
        });

        Ok((block, true))
    }
}
