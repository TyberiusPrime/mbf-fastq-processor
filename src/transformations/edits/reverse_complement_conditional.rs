#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{NewLocation, filter_tag_locations};
use crate::{
    config::{Segment, SegmentIndex},
    dna::HitRegion,
};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReverseComplementConditional {
    /// The tag name containing the boolean value that determines whether to reverse complement
    in_label: String,

    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
}

impl Step for ReverseComplementConditional {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Bool])])
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self.segment_index.unwrap();

        // Apply reverse complement only to reads where the tag is true
        let segment_idx = segment_index.get_index();
        block.segments[segment_idx].apply_mut_with_tag(
            &block.tags,
            &self.in_label,
            |read, tag_val| {
                if tag_val.truthy_val() {
                    read.reverse_complement();
                }
            },
        );

        // Collect tag values for use in the closure
        let tag_values: Vec<bool> = block
            .tags
            .get(&self.in_label)
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tv| tv.truthy_val())
            .collect();

        // Update tag locations for reads where reverse complement was applied
        filter_tag_locations(
            &mut block,
            segment_index,
            |location: &HitRegion, pos, seq: &BString, read_len: usize| -> NewLocation {
                // Check if this read position had reverse complement applied
                if tag_values[pos] {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            segment_index: location.segment_index,
                        },
                        new_seq.into(),
                    )
                } else {
                    NewLocation::Keep
                }
            },
        );

        Ok((block, true))
    }
}
