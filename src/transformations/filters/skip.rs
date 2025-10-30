#![allow(clippy::unnecessary_wraps)] //eserde false positives

use super::super::Step;
use crate::demultiplex::Demultiplex;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Skip {
    pub n: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub so_far: usize,
}

impl Step for Skip {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            Ok((block, true))
        } else if remaining >= block.len() {
            self.so_far += block.len();
            Ok((block.empty(), true))
        } else {
            let here = remaining.min(block.len());
            self.so_far += here;
            block.drain(0..here);
            Ok((block, true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
