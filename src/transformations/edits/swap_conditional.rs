#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{filter_tag_locations_all_targets, NewLocation};
use crate::{
    config::{Segment, SegmentIndex},
    dna::HitRegion,
};

use super::swap::validate_swap_segments;

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
        Some(vec![(
            self.in_label.clone(),
            &[
                TagValueType::Bool,
                TagValueType::String,
                TagValueType::Location,
            ],
        )])
    }

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
                // need to convert to owned, so cloning is fine(ish)
                let temp = block.segments[index_a].entries[read_idx]
                    .to_owned(&block.segments[index_a].block);
                block.segments[index_a].entries[read_idx] = block.segments[index_b].entries
                    [read_idx]
                    .to_owned(&block.segments[index_b].block);
                block.segments[index_b].entries[read_idx] = temp;
            }
        }

        // Update tag locations for all reads where swap occurred
        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, pos: usize| -> NewLocation {
                // Check if this read position was swapped
                if tag_values[pos] {
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
