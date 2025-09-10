#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    NewLocation, Step, Segment, Transformation, apply_in_place, filter_tag_locations,
};
use crate::demultiplex::Demultiplexed;
use crate::dna::HitRegion;
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    #[eserde(compat)]
    segment: Segment,
}

impl Step for CutStart {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        self.segment.validate(input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(&self.segment, |read| read.cut_start(self.n), &mut block);

        filter_tag_locations(
            &mut block,
            &self.segment,
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment: location.segment.clone(),
                    })
                }
            },
        );

        (block, true)
    }
}
