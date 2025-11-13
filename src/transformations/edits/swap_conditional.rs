#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{NewLocation, filter_tag_locations_all_targets};
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Input;
    use crate::io::reads::{FastQBlock, FastQElement, FastQRead, Position};
    use crate::transformations::prelude::InputInfo;
    use std::collections::HashMap;

    /// Helper to create a test block with specified number of reads
    fn create_test_block(
        num_reads: usize,
        segment_a_prefix: &str,
        segment_b_prefix: &str,
    ) -> FastQBlocksCombined {
        let mut segments = vec![FastQBlock::empty(), FastQBlock::empty()];

        for i in 0..num_reads {
            // Segment A
            let name_a = format!("{}_{}", segment_a_prefix, i);
            let seq_a = format!("{}AAAA", i);
            let qual_a = "IIII".to_string();

            segments[0].entries.push(FastQRead {
                name: FastQElement::Owned(name_a.into_bytes()),
                seq: FastQElement::Owned(seq_a.into_bytes()),
                qual: FastQElement::Owned(qual_a.into_bytes()),
            });

            // Segment B
            let name_b = format!("{}_{}", segment_b_prefix, i);
            let seq_b = format!("{}CCCC", i);
            let qual_b = "JJJJ".to_string();

            segments[1].entries.push(FastQRead {
                name: FastQElement::Owned(name_b.into_bytes()),
                seq: FastQElement::Owned(seq_b.into_bytes()),
                qual: FastQElement::Owned(qual_b.into_bytes()),
            });
        }

        FastQBlocksCombined {
            segments,
            output_tags: None,
            tags: HashMap::new(),
            is_final: false,
        }
    }

    /// Helper to add boolean tag to block
    fn add_bool_tag(block: &mut FastQBlocksCombined, label: &str, values: Vec<bool>) {
        block.tags.insert(
            label.to_string(),
            values.into_iter().map(TagValue::Bool).collect(),
        );
    }

    /// Helper to verify swap results
    fn verify_swap(
        block: &FastQBlocksCombined,
        expected_swaps: &[bool],
        segment_a_prefix: &str,
        segment_b_prefix: &str,
    ) {
        for (i, &should_be_swapped) in expected_swaps.iter().enumerate() {
            let seg_a_name = block.segments[0].entries[i]
                .name
                .get(&block.segments[0].block);
            let seg_b_name = block.segments[1].entries[i]
                .name
                .get(&block.segments[1].block);

            if should_be_swapped {
                // Names should be swapped
                assert!(
                    std::str::from_utf8(seg_a_name)
                        .unwrap()
                        .starts_with(segment_b_prefix),
                    "Read {} in segment A should have segment B name after swap",
                    i
                );
                assert!(
                    std::str::from_utf8(seg_b_name)
                        .unwrap()
                        .starts_with(segment_a_prefix),
                    "Read {} in segment B should have segment A name after swap",
                    i
                );
            } else {
                // Names should NOT be swapped
                assert!(
                    std::str::from_utf8(seg_a_name)
                        .unwrap()
                        .starts_with(segment_a_prefix),
                    "Read {} in segment A should keep segment A name",
                    i
                );
                assert!(
                    std::str::from_utf8(seg_b_name)
                        .unwrap()
                        .starts_with(segment_b_prefix),
                    "Read {} in segment B should keep segment B name",
                    i
                );
            }
        }
    }

    #[test]
    fn test_swap_conditional_no_swaps() {
        // All false - no swaps should occur
        let mut block = create_test_block(10, "readA", "readB");
        add_bool_tag(&mut block, "swap", vec![false; 10]);

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        // Verify no swaps occurred
        verify_swap(&block, &vec![false; 10], "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_all_swaps() {
        // All true - all swaps should occur
        let mut block = create_test_block(10, "readA", "readB");
        add_bool_tag(&mut block, "swap", vec![true; 10]);

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        // Verify all swaps occurred
        verify_swap(&block, &vec![true; 10], "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_less_than_half() {
        // 4 out of 10 = 40% - should use normal swap (not block swap)
        let mut block = create_test_block(10, "readA", "readB");
        let swap_pattern = vec![
            true, false, true, false, false, false, true, false, true, false,
        ];
        add_bool_tag(&mut block, "swap", swap_pattern.clone());

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        // Verify swaps match pattern
        verify_swap(&block, &swap_pattern, "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_exactly_half() {
        // 5 out of 10 = 50% - should use normal swap (not block swap, only > 50% triggers it)
        let mut block = create_test_block(10, "readA", "readB");
        let swap_pattern = vec![
            true, false, true, false, true, false, true, false, true, false,
        ];
        add_bool_tag(&mut block, "swap", swap_pattern.clone());

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        // Verify swaps match pattern
        verify_swap(&block, &swap_pattern, "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_more_than_half() {
        // 7 out of 10 = 70% - should use block swap optimization
        let mut block = create_test_block(10, "readA", "readB");
        let swap_pattern = vec![
            true, true, true, false, true, true, true, false, true, false,
        ];
        add_bool_tag(&mut block, "swap", swap_pattern.clone());

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        // Verify swaps match pattern (should use block swap + swap back minority)
        verify_swap(&block, &swap_pattern, "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_single_read_true() {
        // Edge case: single read that should swap
        let mut block = create_test_block(1, "readA", "readB");
        add_bool_tag(&mut block, "swap", vec![true]);

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        verify_swap(&block, &vec![true], "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_single_read_false() {
        // Edge case: single read that should not swap
        let mut block = create_test_block(1, "readA", "readB");
        add_bool_tag(&mut block, "swap", vec![false]);

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        verify_swap(&block, &vec![false], "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_boundary_51_percent() {
        // 51 out of 100 = 51% - just over half, should trigger block swap
        let mut block = create_test_block(100, "readA", "readB");
        let mut swap_pattern = vec![false; 100];
        for i in 0..51 {
            swap_pattern[i] = true;
        }
        add_bool_tag(&mut block, "swap", swap_pattern.clone());

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        verify_swap(&block, &swap_pattern, "readA", "readB");
    }

    #[test]
    fn test_swap_conditional_boundary_49_percent() {
        // 49 out of 100 = 49% - just under half, should use normal swap
        let mut block = create_test_block(100, "readA", "readB");
        let mut swap_pattern = vec![false; 100];
        for i in 0..49 {
            swap_pattern[i] = true;
        }
        add_bool_tag(&mut block, "swap", swap_pattern.clone());

        let mut step = SwapConditional {
            in_label: "swap".to_string(),
            segment_a: None,
            segment_a_index: Some(SegmentIndex(0)),
            segment_b: None,
            segment_b_index: Some(SegmentIndex(1)),
        };

        let input_info = InputInfo::default();
        let result = step.apply(block, &input_info, 0, &None).unwrap();
        let block = result.0;

        verify_swap(&block, &swap_pattern, "readA", "readB");
    }
}
