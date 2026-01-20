#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use super::super::{get_bool_vec_from_tag, ConditionalTag};
use crate::dna::TagValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseType {
    Lower,
    Upper,
}

impl Default for CaseType {
    fn default() -> Self {
        CaseType::Lower
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct _ChangeCase {
    #[serde(alias = "segment")]
    #[serde(alias = "source")]
    target: String,

    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSourceAll>,

    #[serde(default)]
    #[serde(skip)]
    case_type: CaseType,

    #[serde(default)]
    pub if_tag: Option<String>,
}

impl _ChangeCase {
    pub fn new(target: String, case_type: CaseType, if_tag: Option<String>) -> Self {
        Self {
            target,
            case_type,
            resolved_source: None,
            if_tag,
        }
    }
}

impl Step for _ChangeCase {
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

        if tags.is_empty() {
            None
        } else {
            Some(tags)
        }
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

        let case_converter: fn(u8) -> u8 = match self.case_type {
            CaseType::Lower => |b| b.to_ascii_lowercase(),
            CaseType::Upper => |b| b.to_ascii_uppercase(),
        };

        match resolved_source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        let seq = read.seq().to_vec();
                        let new_seq: Vec<u8> = seq.iter().map(|&b| case_converter(b)).collect();
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
                                        case_converter(hit_region.sequence[ii]);
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
                        let new: Vec<u8> = read
                            .name_without_comment(*split_character)
                            .to_vec()
                            .into_iter()
                            .map(case_converter)
                            .collect();
                        if let Some(comment) = read.name_only_comment(*split_character) {
                            let mut full = new;
                            full.push(*split_character);
                            full.extend(comment);
                            read.replace_name(&full);
                        } else {
                            read.replace_name(&new);
                        }
                    },
                    condition.as_deref(),
                );
            }
        }

        Ok((block, true))
    }
}
