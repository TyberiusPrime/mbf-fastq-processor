#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::Demultiplex;

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
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }
        Ok((block, true))
    }
}
