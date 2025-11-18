#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::{ConditionalTag, get_bool_vec_from_tag, prelude::*};

use super::super::{NewLocation, filter_tag_locations_all_targets};
use crate::{
    config::{Segment, SegmentIndex},
    dna::HitRegion,
};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Swap {
    #[serde(default)]
    if_tag: Option<String>,

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
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            vec![(
                cond_tag.tag.clone(),
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )]
        })
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        (
            self.segment_a,
            self.segment_b,
            self.segment_a_index,
            self.segment_b_index,
        ) = validate_swap_segments(self.segment_a.as_ref(), self.segment_b.as_ref(), input_def)?;
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

        // If no condition, do unconditional swap
        if self.if_tag.is_none() {
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

            return Ok((block, true));
        }

        // Conditional swap logic
        let cond_tag = ConditionalTag::from_string(self.if_tag.as_ref().unwrap().clone());
        let tag_values = get_bool_vec_from_tag(&block, &cond_tag);

        // Count how many swaps are needed
        let swap_count = tag_values.iter().filter(|&&x| x).count();
        let total_count = tag_values.len();

        // Optimization: if more than half need swapping, swap the blocks first
        // then swap back the minority
        let (swap_these, did_block_swap) = if swap_count > total_count / 2 {
            // Swap the entire blocks and entries
            block.segments.swap(index_a, index_b);
            // Now we need to swap back the reads that should NOT have been swapped
            (tag_values.iter().map(|&x| !x).collect::<Vec<bool>>(), true)
        } else {
            // Keep the original approach - swap the minority
            (tag_values.clone(), false)
        };

        // Swap individual reads using the optimized swap_with method
        for (read_idx, &should_swap) in swap_these.iter().enumerate() {
            if should_swap {
                // Get mutable references to both blocks for swapping
                let (block_a, block_b) = if index_a < index_b {
                    let (left, right) = block.segments.split_at_mut(index_b);
                    (&mut left[index_a], &mut right[0])
                } else {
                    let (left, right) = block.segments.split_at_mut(index_a);
                    (&mut right[0], &mut left[index_b])
                };

                // Swap the FastQRead entries between the two segments for this read
                block_a.entries[read_idx].swap_with(
                    &mut block_b.entries[read_idx],
                    &mut block_a.block,
                    &mut block_b.block,
                );
            }
        }

        // Update tag locations for all reads where swap occurred
        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, pos: usize| -> NewLocation {
                // Check if this read position was swapped
                // If we did a block swap, the logic is inverted
                let was_swapped = if did_block_swap {
                    // Block was swapped, so all reads are swapped unless they're in swap_these
                    !swap_these[pos]
                } else {
                    // Normal case: only reads in swap_these were swapped
                    swap_these[pos]
                };

                if was_swapped {
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

#[allow(clippy::similar_names)]
#[allow(clippy::type_complexity)]
pub fn validate_swap_segments(
    segment_a: Option<&Segment>,
    segment_b: Option<&Segment>,
    input_def: &crate::config::Input,
) -> Result<(
    Option<Segment>,
    Option<Segment>,
    Option<SegmentIndex>,
    Option<SegmentIndex>,
)> {
    let segment_count = input_def.segment_count();
    if let (Some(seg_a), Some(seg_b)) = (segment_a, segment_b) {
        if seg_a == seg_b {
            bail!("Swap was supplied the same segment for segment_a and segment_b. Please specify two different segments to swap.");
        }
        return Ok((
            segment_a.cloned(),
            segment_b.cloned(),
            Some(seg_a.validate(input_def)?),
            Some(seg_b.validate(input_def)?),
        ));
    }
    if segment_a.is_none() && segment_b.is_none() {
        if segment_count != 2 {
            bail!(
                "Swap requires exactly 2 input segments when segment_a and segment_b are omitted, but {segment_count} segments were provided. Either specify segment_a and segment_b explicitly, or use exactly 2 input segments for auto-detection.",
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
        "Swap requires both segment_a and segment_b to be specified, or both to be omitted for auto-detection with exactly 2 segments. Please either specify both segments, or omit both for auto-detection."
    );
}
