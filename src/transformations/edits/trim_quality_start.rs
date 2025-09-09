#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    NewLocation, Step, Target, Transformation, apply_in_place_wrapped,
    filter_tag_locations, validate_target,
};
use crate::{
    config::deser::u8_from_char_or_number,
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimQualityStart {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}

impl Step for TrimQualityStart {
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
        let mut cut_off = Vec::new();
        {
            let edit_cut_off = &mut cut_off;
            apply_in_place_wrapped(
                self.target,
                |read| {
                    let read_len = read.len();
                    read.trim_quality_start(self.min);
                    let lost = read_len - read.len();
                    edit_cut_off.push(lost);
                },
                &mut block,
            );
        }

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, pos, _seq, _read_len: usize| -> NewLocation {
                let lost = cut_off[pos];
                if location.start < lost {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - lost,
                        len: location.len,
                        target: location.target,
                    })
                }
            },
        );

        (block, true)
    }
}