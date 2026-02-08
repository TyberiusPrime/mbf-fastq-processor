#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

/// remove one tag from memory

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ForgetTag {
    in_label: String,
}

impl Step for ForgetTag {
    fn removes_tags(&self) -> Vec<String> {
        vec![self.in_label.clone()]
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.tags.remove(&self.in_label);
        Ok((block, true))
    }
}
