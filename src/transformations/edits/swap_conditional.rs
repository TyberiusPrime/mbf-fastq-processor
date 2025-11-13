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
pub struct SwapConditional {
    /// The tag name containing the boolean value that determines whether to swap
    in_label: String,

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

impl Step for SwapConditional {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Bool])])
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        let segment_count = input_def.segment_count();

        // Case 1: Both segments specified explicitly
        if let (Some(seg_a), Some(seg_b)) = (&self.segment_a, &self.segment_b) {
            if seg_a == seg_b {
                bail!("SwapConditional was supplied the same segment for segment_a and segment_b");
            }
            self.segment_a_index = Some(seg_a.clone().validate(input_def)?);
            self.segment_b_index = Some(seg_b.clone().validate(input_def)?);
            return Ok(());
        }

        // Case 2: Auto-detect for exactly two segments
        if self.segment_a.is_none() && self.segment_b.is_none() {
            if segment_count != 2 {
                bail!(
                    "SwapConditional requires exactly 2 input segments when segment_a and segment_b are omitted, but {segment_count} segments were provided",
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
            "SwapConditional requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments"
        );
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

        // Collect the boolean tag values to avoid borrowing issues
        let tag_values: Vec<bool> = block
            .tags
            .get(&self.in_label)
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tv| tv.truthy_val())
            .collect();

        // Swap individual reads where the tag is true
        for (read_idx, &should_swap) in tag_values.iter().enumerate() {
            if should_swap {
                // Swap the FastQRead entries between the two segments for this read
                let temp = block.segments[index_a].entries[read_idx].clone();
                block.segments[index_a].entries[read_idx] =
                    block.segments[index_b].entries[read_idx].clone();
                block.segments[index_b].entries[read_idx] = temp;
            }
        }

        // Update tag locations for all reads where swap occurred
        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, pos: usize| -> NewLocation {
                // Check if this read position was swapped
                if pos < tag_values.len() && tag_values[pos] {
                    NewLocation::New(HitRegion {
                        start: location.start,
                        len: location.len,
                        segment_index: match location.segment_index {
                            SegmentIndex(index) if index == index_a => SegmentIndex(index_b),
                            SegmentIndex(index) if index == index_b => SegmentIndex(index_a),
                            _ => location.segment_index, // others unchanged
                        },
                    })
                } else {
                    NewLocation::Keep
                }
            },
        );

        Ok((block, true))
    }
}
