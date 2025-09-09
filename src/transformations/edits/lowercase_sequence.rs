#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    Step, Transformation, apply_in_place_wrapped_plus_all, validate_target_plus_all,
};
use crate::{config::TargetPlusAll, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseSequence {
    target: TargetPlusAll,
}

impl Step for LowercaseSequence {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target_plus_all(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_plus_all(
            self.target,
            |read| {
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
        );

        (block, true)
    }
}
