#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;

use super::super::{apply_filter, validate_target, Step, Target, Transformation};
use crate::demultiplex::Demultiplexed;
use serde_valid::Validate;

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct LowComplexity {
    pub target: Target,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub threshold: f32,
}

impl Step for LowComplexity {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| {
            // Calculate the number of transitions
            let mut transitions = 0;
            let seq = read.seq();
            for ii in 0..seq.len() - 1 {
                if seq[ii] != seq[ii + 1] {
                    transitions += 1;
                }
            }
            let ratio = transitions as f32 / (read.len() - 1) as f32;
            ratio >= self.threshold
        });
        (block, true)
    }
}