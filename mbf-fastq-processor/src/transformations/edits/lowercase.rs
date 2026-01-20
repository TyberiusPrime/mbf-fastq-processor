#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use super::super::{ConditionalTag, get_bool_vec_from_tag};
use crate::dna::TagValue;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Lowercase {
    #[serde(alias = "segment")]
    #[serde(alias = "source")]
    target: String,

    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSourceAll>,

    #[serde(default)]
    if_tag: Option<String>,
}

impl Step for Lowercase {
    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        let mut tags = Vec::new();

        if let Some(ref resolved) = self.resolved_source {
            if let Some(resolved_tags) = resolved.get_tags() {
                tags.extend(resolved_tags);
            }
        }

        if let Some(ref tag_str) = self.if_tag {
            let cond_tag = ConditionalTag::from_string(tag_str.clone());
            tags.push((
                cond_tag.tag.clone(),
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            ));
        }

        if tags.is_empty() { None } else { Some(tags) }
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.resolved_source = Some(ResolvedSourceAll::parse(&self.target, input_def)?);
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

        let resolved_source = self
            .resolved_source
            .as_ref()
            .expect("resolved_source must be set during initialization");

        match resolved_source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        let seq = read.seq().to_vec();
                        let new_seq: Vec<u8> =
                            seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                        read.replace_seq_keep_qual(&new_seq);
                    },
                    condition.as_deref(),
                );
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    for tag_val in hits.iter_mut() {
                        if let TagValue::Location(hit) = tag_val {
                            for hit_region in &mut hit.0 {
                                for ii in 0..hit_region.sequence.len() {
                                    hit_region.sequence[ii] =
                                        hit_region.sequence[ii].to_ascii_lowercase();
                                }
                            }
                        }
                    }
                }
            }
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        let mut new = read
                            .name_without_comment(*split_character)
                            .to_vec()
                            .to_ascii_lowercase();
                        if let Some(comment) = read.name_only_comment(*split_character) {
                            new.push(*split_character);
                            new.extend(comment)
                        }
                        read.replace_name(&new);
                    },
                    condition.as_deref(),
                );
            }
        }

        Ok((block, true))
    }
}
