#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByTag {
    label: String,
    keep_or_remove: super::super::KeepOrRemove,
}

impl Step for ByTag {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(
            self.label.clone(),
            &[
                TagValueType::Location,
                TagValueType::String,
                TagValueType::Bool,
            ],
        )])
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut keep: Vec<bool> = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tag_val| tag_val.truthy_val())
            .collect();
        if self.keep_or_remove == super::super::KeepOrRemove::Remove {
            keep.iter_mut().for_each(|x| *x = !*x); //flip
        }
        super::super::apply_bool_filter(&mut block, &keep);

        Ok((block, true))
    }
}
