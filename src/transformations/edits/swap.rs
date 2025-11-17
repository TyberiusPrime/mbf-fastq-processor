#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{NewLocation, filter_tag_locations_all_targets};
use crate::{
    config::{Segment, SegmentIndex},
    dna::HitRegion,
};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
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
        (
            self.segment_a,
            self.segment_b,
            self.segment_a_index,
            self.segment_b_index,
        ) = validate_swap_segments(&self.segment_a, &self.segment_b, input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
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
                        _ => location.segment_index, // others unchanged
                    },
                })
            },
        );

        Ok((block, true))
    }
}

#[allow(clippy::similar_names)]
pub fn validate_swap_segments(
    segment_a: &Option<Segment>,
    segment_b: &Option<Segment>,
    input_def: &crate::config::Input,
) -> Result<
    (
        Option<Segment>,
        Option<Segment>,
        Option<SegmentIndex>,
        Option<SegmentIndex>,
    ),
    anyhow::Error,
> {
    let segment_count = input_def.segment_count();
    if let (Some(seg_a), Some(seg_b)) = (segment_a, segment_b) {
        if seg_a == seg_b {
            bail!("Swap was supplied the same segment for segment_a and segment_b");
        }
        return Ok((
            segment_a.clone(),
            segment_b.clone(),
            Some(seg_a.validate(input_def)?),
            Some(seg_b.validate(input_def)?),
        ));
    }
    if segment_a.is_none() && segment_b.is_none() {
        if segment_count != 2 {
            bail!(
                "Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but {segment_count} segments were provided",
            );
        }

        let segment_order = input_def.get_segment_order();
        let seg_a = Segment(segment_order[0].clone());
        let seg_b = Segment(segment_order[1].clone());

        let segment_a_index = Some(seg_a.validate(input_def)?);
        let segment_b_index = Some(seg_b.validate(input_def)?);
        return Ok((Some(seg_a), Some(seg_b), segment_a_index, segment_b_index));
    }
    bail!(
        "Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments"
    );
}
impl Swap {}
