#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;

use super::super::{
    apply_filter, apply_filter_all, validate_target_plus_all, Step, TargetPlusAll, Transformation,
};
use crate::demultiplex::Demultiplexed;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct TooManyN {
    pub target: TargetPlusAll,
    pub n: usize,
}
impl Step for TooManyN {
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
        if let Ok(target) = self.target.try_into() {
            apply_filter(target, &mut block, |read| {
                let seq = read.seq();
                let sum: usize = seq.iter().map(|x| usize::from(*x == b'N')).sum();
                sum <= self.n
            });
            return (block, true);
        } else {
            apply_filter_all(&mut block, |read1, read2, index1, index2| {
                let mut sum = 0;
                for seq in [read1.seq()]
                    .iter()
                    .chain(read2.as_ref().map(|r| r.seq()).iter())
                    .chain(index1.as_ref().map(|r| r.seq()).iter())
                    .chain(index2.as_ref().map(|r| r.seq()).iter())
                {
                    sum += seq.iter().map(|x| usize::from(*x == b'N')).sum::<usize>();
                    if sum > self.n {
                        return false;
                    }
                }
                true
            });
        }
        (block, true)
    }
}