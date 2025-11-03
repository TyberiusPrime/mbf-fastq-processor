#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ForgetTag {
    label: String,
}

impl Step for ForgetTag {
    fn removes_tags(&self) -> Option<Vec<String>> {
        Some(vec![self.label.clone()])
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }
        Ok((block, true))
    }
}
