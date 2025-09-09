#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::Demultiplexed;

use super::super::Step;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RemoveTag {
    label: String,
}

impl Step for RemoveTag {
    fn removes_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }
        (block, true)
    }
}
