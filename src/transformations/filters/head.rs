#![allow(clippy::unnecessary_wraps)] //eserde false positives

use super::super::Step;
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub n: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub so_far: usize,
}

impl Step for Head {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            Ok((block.empty(), false))
        } else {
            block.resize(remaining.min(block.len()));
            let do_continue = remaining > block.len();
            self.so_far += block.len();
            Ok((block, do_continue))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
