#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::{ConditionalTag, get_bool_vec_from_tag};
use crate::config::{
    Segment, SegmentIndex,
    deser::{bstring_from_string, dna_from_string},
};
use bstr::BString;

/// Add a fixed sequence to the end of reads
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Postfix {
    #[serde(default)]
    pub segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    #[serde(deserialize_with = "dna_from_string")]
    #[schemars(with = "String")]
    pub seq: BString,
    #[serde(deserialize_with = "bstring_from_string")]
    //we don't check the quality. It's on you if you
    //write non phred values in there
    #[schemars(with = "String")]
    pub qual: BString,

    #[serde(default)]
    if_tag: Option<String>,
}

impl Step for Postfix {
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

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!(
                "Postfix: 'seq' and 'qual' must be the same length. Sequence has {} characters but quality string has {} characters. Please ensure they match.",
                self.seq.len(),
                self.qual.len()
            );
        }
        Ok(())
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

        block.apply_in_place_wrapped(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            |read| read.postfix(&self.seq, &self.qual),
            condition.as_deref(),
        );
        // postfix doesn't change tags.
        Ok((block, true))
    }
}
