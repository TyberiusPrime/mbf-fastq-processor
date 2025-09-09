#![allow(clippy::unnecessary_wraps)] //eserde false positives

use super::super::Step;
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

impl Step for Head {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            (block.empty(), false)
        } else {
            block.resize(remaining.min(block.len()));
            let do_continue = remaining > block.len();
            self.so_far += block.len();
            (block, do_continue)
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
