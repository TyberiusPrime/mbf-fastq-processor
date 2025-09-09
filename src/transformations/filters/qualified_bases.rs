#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;

use super::super::{Step, Target, Transformation, apply_filter, validate_target};
use crate::{config::deser::u8_from_char_or_number, demultiplex::Demultiplexed};
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct QualifiedBases {
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub min_ratio: f32,
    pub target: Target,
}

impl Step for QualifiedBases {
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
            let qual = read.qual();
            let sum: usize = qual
                .iter()
                .map(|x| usize::from(*x >= self.min_quality))
                .sum();
            let pct = sum as f32 / qual.len() as f32;
            pct >= self.min_ratio
        });
        (block, true)
    }
}
