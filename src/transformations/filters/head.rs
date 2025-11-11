#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
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
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            let mut empty = block.empty();
            empty.is_final = true;
            Ok((empty, false))
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
