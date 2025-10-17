#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::{demultiplex::Demultiplexed, dna::TagValue, transformations::TagValueType};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseTag {
    label: String,
}

impl Step for LowercaseTag {
    fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
        vec![(self.label.clone(), TagValueType::Location)].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let hits = block
            .tags
            .as_mut()
            .and_then(|tags| tags.get_mut(&self.label))
            .expect("Tag missing. Should been caught earlier.");
        for tag_val in hits.iter_mut() {
            if let TagValue::Sequence(hit) = tag_val {
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
