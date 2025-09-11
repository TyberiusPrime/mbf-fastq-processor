#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{filter_tag_locations_all_targets, NewLocation, Step};
use crate::{
    config::{Segment, SegmentIndex},
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::{Result};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwapR1AndR2 {
    segment_a: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_a_index: Option<SegmentIndex>,

    segment_b: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_b_index: Option<SegmentIndex>,
}

impl Step for SwapR1AndR2 {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_a_index = Some(self.segment_a.validate(input_def)?);
        self.segment_b_index = Some(self.segment_b.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let index_a = self.segment_a_index.as_ref().unwrap().get_index();
        let index_b = self.segment_b_index.as_ref().unwrap().get_index();
        let name_a = self
            .segment_a_index
            .as_ref()
            .unwrap()
            .get_name()
            .to_string();
        let name_b = self
            .segment_b_index
            .as_ref()
            .unwrap()
            .get_name()
            .to_string();
        block.segments.swap(index_a, index_b);

        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, _pos: usize| -> NewLocation {
                NewLocation::New(HitRegion {
                    start: location.start,
                    len: location.len,
                    segment_index: match location.segment_index {
                        SegmentIndex(index, _) if index == index_a => SegmentIndex(index_b, name_b.clone()),
                        SegmentIndex(index, _) if index == index_b => SegmentIndex(index_a, name_a.clone()),
                        _ => location.segment_index.clone(), // others unchanged
                    },
                })
            },
        );

        (block, true)
    }
}
