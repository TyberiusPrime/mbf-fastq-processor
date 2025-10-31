use crate::transformations::prelude::*;

use super::super::KeepOrRemove;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByBoolTag {
    pub label: String,
    pub keep_or_remove: KeepOrRemove,
}

impl Step for ByBoolTag {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.label.clone(), &[TagValueType::Bool])])
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let tag_values = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Bool tag not found");

        let keep: Vec<bool> = tag_values
            .iter()
            .map(|tag_val| tag_val.as_bool().unwrap_or_default())
            .map(|passes| {
                if self.keep_or_remove == KeepOrRemove::Remove {
                    !passes
                } else {
                    passes
                }
            })
            .collect();

        super::super::apply_bool_filter(&mut block, &keep);
        Ok((block, true))
    }
}
