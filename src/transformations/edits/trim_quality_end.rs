#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    Step, Target, Transformation, apply_in_place_wrapped,
    filter_tag_locations_beyond_read_length, validate_target,
};
use crate::{
    config::deser::u8_from_char_or_number,
    demultiplex::Demultiplexed,
};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimQualityEnd {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}

impl Step for TrimQualityEnd {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.target,
            |read| read.trim_quality_end(self.min),
            &mut block,
        );
        filter_tag_locations_beyond_read_length(&mut block, self.target);
        (block, true)
    }
}