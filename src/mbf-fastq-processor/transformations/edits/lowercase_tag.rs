#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::TagValue;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LowercaseTag {
    in_label: String,
}

impl Step for LowercaseTag {
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
        let hits = block
            .tags
            .get_mut(&self.in_label)
            .expect("Tag missing. Should been caught earlier.");
        for tag_val in hits.iter_mut() {
            if let TagValue::Location(hit) = tag_val {
                for hit_region in &mut hit.0 {
                    for ii in 0..hit_region.sequence.len() {
                        hit_region.sequence[ii] = hit_region.sequence[ii].to_ascii_lowercase();
                    }
                }
            }
        }

        Ok((block, true))
    }
}
