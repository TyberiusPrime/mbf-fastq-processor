#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

/// Remove all tags from memory

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct ForgetAllTags {
    ignored: Option<u8>, //tdp dislikes empty structs
}

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
