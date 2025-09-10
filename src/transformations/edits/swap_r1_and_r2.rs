#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{filter_tag_locations_all_targets, NewLocation, Segment, Step, Transformation};
use crate::{demultiplex::Demultiplexed, dna::HitRegion};
use anyhow::{bail, Result};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwapR1AndR2 {
    segment_a: Segment,
    segment_b: Segment,
}

impl Step for SwapR1AndR2 {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        self.segment_a.validate(input_def)?;
        self.segment_b.validate(input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        todo!();
        /* let read1 = block.read1;
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
                                Segment::Read1 => Segment::Read2,
                                Segment::Read2 => Segment::Read1,
                                _ => location.target, // Indexes remain unchanged
                            },
                        })
                    },
                );

                (block, true)
        */
    }
}
