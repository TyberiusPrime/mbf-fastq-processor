#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{
    ConditionalTag, NewLocation, apply_in_place, filter_tag_locations, get_bool_vec_from_tag,
};
use crate::config::{Segment, SegmentIndex};
use crate::dna::HitRegion;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CutStart {
    n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
    #[serde(default)]
    if_tag: Option<String>,
}

impl Step for CutStart {
    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
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

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self.if_tag.as_ref().map(|tag_str| {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            get_bool_vec_from_tag(&block, &cond_tag)
        });

        apply_in_place(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.cut_start(self.n),
            &mut block,
            condition.as_deref(),
        );

        filter_tag_locations(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                if location.start < self.n {
                    NewLocation::Remove
                } else {
                    NewLocation::New(HitRegion {
                        start: location.start - self.n,
                        len: location.len,
                        segment_index: location.segment_index,
                    })
                }
            },
            condition.as_deref(),
        );

        Ok((block, true))
    }
}
