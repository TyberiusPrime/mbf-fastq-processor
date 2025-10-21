#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ForgetAllTags {}

impl Step for ForgetAllTags {
    fn removes_all_tags(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        block.tags = None;
        Ok((block, true))
    }
}
