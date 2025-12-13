#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::super::{ConditionalTag, apply_in_place_wrapped_plus_all, get_bool_vec_from_tag};
use crate::config::{SegmentIndexOrAll, SegmentOrAll};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LowercaseSequence {
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
    #[serde(default)]
    if_tag: Option<String>,
}

impl Step for LowercaseSequence {
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

        apply_in_place_wrapped_plus_all(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| {
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
            condition.as_deref(),
        );

        Ok((block, true))
    }
}
