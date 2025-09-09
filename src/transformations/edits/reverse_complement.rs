#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    NewLocation, Step, Target, Transformation, apply_in_place_wrapped, filter_tag_locations,
    validate_target,
};
use crate::{demultiplex::Demultiplexed, dna::HitRegion};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReverseComplement {
    pub target: Target,
}

impl Step for ReverseComplement {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(self.target, |read| read.reverse_complement(), &mut block);

        filter_tag_locations(
            &mut block,
            self.target,
            |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            target: location.target,
                        },
                        new_seq.into(),
                    )
                }
            },
        );

        (block, true)
    }
}
