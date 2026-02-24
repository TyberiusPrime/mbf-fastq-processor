#![allow(clippy::struct_field_names)]
#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::HitRegion;

/// Swap two segments
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Swap {
    #[tpd(default)]
    if_tag: Option<String>,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment_a: SegmentIndex,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment_b: SegmentIndex,
}

impl VerifyIn<PartialConfig> for PartialSwap {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        let input_def = parent
            .input
            .as_ref()
            .expect("Input definition must be set before config for Swap step validation");
        let segment_order = input_def.get_segment_order();

        if self.segment_a.is_missing() ^ self.segment_b.is_missing() {
            self.segment_a.state = TomlValueState::Nested;
            self.segment_b.state = TomlValueState::Nested;
            return Err(ValidationFailure::new(
                "Insuffient swap definition",
                Some(
                    "Please either specify both segment_a and segment_b, or omit both for auto-detection.",
                ),
            ));
        } else if self.segment_a.is_missing() && self.segment_b.is_missing() {
            if segment_order.len() == 2 {
                self.segment_a = TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(0)), 0..0);
                self.segment_b = TomlValue::new_ok(MustAdapt::PostVerify(SegmentIndex(1)), 0..0);
            } else {
                self.segment_a.state = TomlValueState::Nested;
                self.segment_b.state = TomlValueState::Nested;
                return Err(ValidationFailure::new(
                    "Insuffient swap definition",
                    Some(
                        "There were more (or fewer) than 2 segments, and you did not specify both segment_a and segment_b.",
                    ),
                ));
            }
        } else if self.segment_a.is_needs_further_validation()
            && self.segment_b.is_needs_further_validation()
        {
            self.segment_a.validate_segment(parent);
            self.segment_b.validate_segment(parent);
            if self.segment_a.is_ok()
                && self.segment_b.is_ok()
                && self
                    .segment_a
                    .as_ref()
                    .expect("just checked is._ok")
                    .as_ref_post()
                    == self
                        .segment_b
                        .as_ref()
                        .expect("just checked is._ok")
                        .as_ref_post()
            {
                let spans = vec![
                    (self.segment_a.span(), "Identical to segment_b".to_string()),
                    (self.segment_b.span(), "Identical to segment_a".to_string()),
                ];
                self.segment_a.state = TomlValueState::Custom { spans };
                self.segment_a.help =
                    Some("Please specify two different segments to swap.".to_string());
                self.segment_b.state = TomlValueState::Nested;
            }
        } else {
            if self.segment_a.is_needs_further_validation() {
                self.segment_a.validate_segment(parent);
            }
            if self.segment_b.is_needs_further_validation() {
                self.segment_b.validate_segment(parent);
            }
            //all other errors we pass straight on
        }
        Ok(())
    }
}

impl Step for Swap {
    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
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

    fn must_see_all_tags(&self) -> bool {
        true
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let index_a = self.segment_a.get_index();
        let index_b = self.segment_b.get_index();

        // If no condition, do unconditional swap
        if self.if_tag.is_none() {
            block.segments.swap(index_a, index_b);

            block.filter_tag_locations_all_targets(
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
        let cond_tag = ConditionalTag::from_string(
            self.if_tag
                .as_ref()
                .expect("if_tag must be set when conditional swap is used")
                .clone(),
        );
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
        //make sure that actually worked.

        // Swap individual reads using the optimized swap_with method
        let mut actual_swap_count = 0;
        for (read_idx, &should_swap) in swap_these.iter().enumerate() {
            if should_swap {
                actual_swap_count += 1;
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
        assert!(actual_swap_count <= total_count / 2); //verify we actually went with the smaller
        //one. Makes mutation testing happy.

        // Update tag locations for all reads where swap occurred
        block.filter_tag_locations_all_targets(|location: &HitRegion, pos: usize| -> NewLocation {
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
                        //todo: make sure per test case this actually works as expected
                        SegmentIndex(index) if index == index_a => SegmentIndex(index_b),
                        SegmentIndex(index) if index == index_b => SegmentIndex(index_a),
                        _ => location.segment_index, // others unchanged
                    },
                })
            } else {
                NewLocation::Keep
            }
        });

        Ok((block, true))
    }
}
