#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForgetTag {
    in_label: String,
}

impl Step for ForgetTag {
    fn removes_tags(&self) -> Option<Vec<String>> {
        Some(vec![self.in_label.clone()])
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.tags.remove(&self.in_label);
        Ok((block, true))
    }
}
