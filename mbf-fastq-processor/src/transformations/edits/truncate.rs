#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{ConditionalTag, get_bool_vec_from_tag};
use crate::config::{Segment, SegmentIndex};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Truncate {
    n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
    #[serde(default)]
    if_tag: Option<String>,
}

impl Step for Truncate {
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

        block.apply_in_place(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.max_len(self.n),
            condition.as_deref(),
        );
        block.filter_tag_locations_beyond_read_length(
            self.segment_index
                .expect("segment_index must be set during initialization"),
        );
        Ok((block, true))
    }
    //
    //to cut location tags
    fn must_see_all_tags(&self) -> bool {
        true
    }
}
