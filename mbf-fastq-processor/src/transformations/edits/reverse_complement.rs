#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{config::SegmentIndex, dna::HitRegion};

/// Reverse complement a read
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReverseComplement {
    #[tpd_default]
    segment: Segment,
    #[schemars(skip)]
    #[tpd(skip)]
    segment_index: Option<SegmentIndex>,

    if_tag: Option<String>,
}

impl Step for ReverseComplement {
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
    //to modify location tags
    fn must_see_all_tags(&self) -> bool {
        true
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            get_bool_vec_from_tag(&block, &cond_tag)
        });

        block.apply_in_place_wrapped(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.reverse_complement(),
            condition.as_deref(),
        );

        block.filter_tag_locations(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                {
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
                }
            },
            condition.as_deref(),
        );

        Ok((block, true))
    }
}

