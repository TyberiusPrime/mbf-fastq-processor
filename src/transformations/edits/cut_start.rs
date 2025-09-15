#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{NewLocation, Step, apply_in_place, filter_tag_locations};
use crate::config::{Segment, SegmentIndex};
use crate::demultiplex::Demultiplexed;
use crate::dna::HitRegion;
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
}

impl Step for CutStart {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        apply_in_place(
            self.segment_index.as_ref().unwrap(),
            |read| read.cut_start(self.n),
            &mut block,
        );

        filter_tag_locations(
            &mut block,
            self.segment_index.as_ref().unwrap(),
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment_index: location.segment_index.clone(),
                    })
                }
            },
        );

        Ok((block, true))
    }
}
