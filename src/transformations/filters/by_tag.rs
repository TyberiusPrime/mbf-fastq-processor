#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{Demultiplexed, transformations::TagValueType};

use super::super::{Step};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByTag {
    label: String,
    keep_or_remove: super::super::KeepOrRemove,
}

impl Step for ByTag {
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
        let mut keep: Vec<bool> = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tag_val| !tag_val.is_missing())
            .collect();
        if self.keep_or_remove == super::super::KeepOrRemove::Remove {
            keep.iter_mut().for_each(|x| *x = !*x);
        }
        super::super::apply_bool_filter(&mut block, &keep);

        Ok((block, true))
    }
}
