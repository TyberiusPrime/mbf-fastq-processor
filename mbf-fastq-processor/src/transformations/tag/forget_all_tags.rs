#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForgetAllTags {}

impl Step for ForgetAllTags {
    fn removes_all_tags(&self) -> bool {
        true
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.tags = Default::default();
        Ok((block, true))
    }
}
