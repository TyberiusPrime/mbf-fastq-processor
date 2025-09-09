#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    NewLocation, Step, Target, Transformation,
    filter_tag_locations_all_targets,
};
use crate::{
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::{Result, bail};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwapR1AndR2 {}

impl Step for SwapR1AndR2 {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        {
            if input_def.read2.is_none() {
                bail!(
                    "Read2 is not defined in the input section, but used by transformation SwapR1AndR2"
                );
            }
            Ok(())
        }
    }
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let read1 = block.read1;
        let read2 = block.read2.take().unwrap();
        block.read1 = read2;
        block.read2 = Some(read1);

        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, _pos: usize| -> NewLocation {
                NewLocation::New(HitRegion {
                    start: location.start,
                    len: location.len,
                    target: match location.target {
                        Target::Read1 => Target::Read2,
                        Target::Read2 => Target::Read1,
                        _ => location.target, // Indexes remain unchanged
                    },
                })
            },
        );

        (block, true)
    }
}