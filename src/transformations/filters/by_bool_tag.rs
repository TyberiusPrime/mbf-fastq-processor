use anyhow::Result;

use super::super::{KeepOrRemove, Step};
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByBoolTag {
    pub label: String,
    pub keep_or_remove: KeepOrRemove,
}

impl Step for ByBoolTag {
    fn uses_tags(&self) -> Option<Vec<String>> {
        Some(vec![self.label.clone()])
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let tag_values = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Bool tag not found");

        let keep: Vec<bool> = tag_values
            .iter()
            .map(|tag_val| {
                if let Some(passes) = tag_val.as_bool() {
                    passes
                } else {
                    panic!("FilterByBoolTag applied to non-boolean tag");
                }
            })
            .map(|passes| {
                if self.keep_or_remove == KeepOrRemove::Remove {
                    !passes
                } else {
                    passes
                }
            })
            .collect();

        super::super::apply_bool_filter(&mut block, &keep);
        (block, true)
    }
}
