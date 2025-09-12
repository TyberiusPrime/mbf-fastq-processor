#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{filter_tag_locations_all_targets, NewLocation, Step};
use crate::{
    config::{Segment, SegmentIndex},
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::{bail, Result};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Swap {
    #[serde(default)]
    segment_a: Option<Segment>,
    #[serde(default)]
    #[serde(skip)]
    segment_a_index: Option<SegmentIndex>,

    #[serde(default)]
    segment_b: Option<Segment>,
    #[serde(default)]
    #[serde(skip)]
    segment_b_index: Option<SegmentIndex>,
}

impl Step for Swap {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        let segment_count = input_def.segment_count();

        // Case 1: Both segments specified explicitly
        if let (Some(seg_a), Some(seg_b)) = (&self.segment_a, &self.segment_b) {
            if seg_a == seg_b {
                bail!("Swap was supplied the same segment for segment_a and segment_b");
            }
            self.segment_a_index = Some(seg_a.clone().validate(input_def)?);
            self.segment_b_index = Some(seg_b.clone().validate(input_def)?);
            return Ok(());
        }

        // Case 2: Auto-detect for exactly two segments
        if self.segment_a.is_none() && self.segment_b.is_none() {
            if segment_count != 2 {
                bail!(
                    "Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but {} segments were provided",
                    segment_count
                );
            }

            let segment_order = input_def.get_segment_order();
            let mut seg_a = Segment(segment_order[0].clone());
            let mut seg_b = Segment(segment_order[1].clone());

            self.segment_a_index = Some(seg_a.validate(input_def)?);
            self.segment_b_index = Some(seg_b.validate(input_def)?);
            self.segment_a = Some(seg_a);
            self.segment_b = Some(seg_b);
            return Ok(());
        }

        // Case 3: Partial specification error
        bail!(
            "Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments"
        );
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let index_a = self.segment_a_index.as_ref().unwrap().get_index();
        let index_b = self.segment_b_index.as_ref().unwrap().get_index();
        block.segments.swap(index_a, index_b);

        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, _pos: usize| -> NewLocation {
                NewLocation::New(HitRegion {
                    start: location.start,
                    len: location.len,
                    segment_index: match location.segment_index {
                        SegmentIndex(index) if index == index_a => SegmentIndex(index_b),
                        SegmentIndex(index) if index == index_b => SegmentIndex(index_a),
                        _ => location.segment_index.clone(), // others unchanged
                    },
                })
            },
        );

        Ok((block, true))
    }
}
