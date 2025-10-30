#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::{Context, Result, bail};
use std::{collections::HashSet, path::Path};

use super::super::extract_bool_tags_plus_all;

use crate::config::{self, Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};
use crate::demultiplex::{Demultiplex, DemultiplexInfo};
use crate::dna::TagValue;
use crate::transformations::{
    FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter, Step,
    read_name_canonical_prefix, reproducible_cuckoofilter,
};
use serde_valid::Validate;

// we settled on the Cuckoofilter after doing experiments/memory_usage_hashset_vs_radis
#[derive(Debug, Validate, Clone)]
pub enum ApproxOrExactFilter {
    Exact(HashSet<Vec<u8>>),
    Approximate(Box<OurCuckCooFilter<FragmentEntryForCuckooFilter>>),
}

impl ApproxOrExactFilter {
    #[allow(dead_code)]
    pub fn contains(&self, seq: &FragmentEntry) -> bool {
        match self {
            ApproxOrExactFilter::Exact(hashset) => hashset.contains(&seq.to_continuous_vec()),
            ApproxOrExactFilter::Approximate(filter) => filter.contains(seq),
        }
    }

    pub fn containsert(&mut self, seq: &FragmentEntry) -> bool {
        match self {
            ApproxOrExactFilter::Exact(hashset) => {
                let q = seq.to_continuous_vec();
                if !hashset.contains(&q) {
                    hashset.insert(q);
                    return false;
                }
                true
            }
            ApproxOrExactFilter::Approximate(filter) => {
                if !filter.contains(seq) {
                    filter.insert(seq);
                    return false;
                }
                true
            }
        }
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, seq: &FragmentEntry) {
        match self {
            ApproxOrExactFilter::Exact(hashset) => {
                hashset.insert(seq.to_continuous_vec());
            }
            ApproxOrExactFilter::Approximate(filter) => {
                filter.insert(seq);
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ResolvedSource {
    Segment(SegmentIndexOrAll),
    Tag(String),
    Name {
        segment: SegmentIndex,
        split_character: u8,
    },
}

fn default_source() -> String {
    SegmentOrAll::default().0
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Duplicates {
    #[serde(default = "default_source")]
    source: String,
    #[serde(default)]
    #[serde(deserialize_with = "config::deser::single_u8_from_string")]
    split_character: Option<u8>,
    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSource>,
    pub label: String,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    pub seed: Option<u64>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,
}
impl Step for Duplicates {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate seed requirement based on false_positive_rate
        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if self.label.starts_with("tag:") || self.label.starts_with("name:") {
            bail!(
                "TagDuplicates label '{label}' must not start with 'name:' or 'tag:'",
                label = self.label
            );
        }

        let source = self.source.trim();
        let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
            let trimmed = tag_name.trim();
            if trimmed.is_empty() {
                bail!("TagDuplicates source tag name may not be empty");
            }
            if self.split_character.is_some() {
                bail!("split_character is only valid when source starts with 'name:'");
            }
            ResolvedSource::Tag(trimmed.to_string())
        } else if let Some(segment_name) = source.strip_prefix("name:") {
            let trimmed = segment_name.trim();
            if trimmed.is_empty() {
                bail!("TagDuplicates name source requires a segment name");
            }
            let mut segment = Segment(trimmed.to_string());
            let segment_index = segment.validate(input_def)?;
            let split_character = self.split_character.context(
                "TagDuplicates using a 'name:' source requires 'split_character' to be set",
            )?;
            ResolvedSource::Name {
                segment: segment_index,
                split_character,
            }
        } else {
            let mut segment = SegmentOrAll(source.to_string());
            if self.split_character.is_some() {
                bail!("split_character is only valid when source starts with 'name:'");
            }
            ResolvedSource::Segment(segment.validate(input_def)?)
        };
        self.resolved_source = Some(resolved);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        match &self.resolved_source {
            Some(ResolvedSource::Tag(tag_name)) => Some(vec![(
                tag_name.clone(),
                &[
                    crate::transformations::TagValueType::String,
                    crate::transformations::TagValueType::Location,
                ],
            )]),
            _ => None,
        }
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplex,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexInfo>> {
        let filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
            ApproxOrExactFilter::Exact(HashSet::new())
        } else {
            let seed = self
                .seed
                .expect("seed should be validated to exist when false_positive_rate > 0.0");
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                seed,
                1_000_000,
                self.false_positive_rate,
            )))
        };
        self.filter = Some(filter);
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let resolved = self
            .resolved_source
            .clone()
            .expect("validate_segments should have populated the source");

        match resolved {
            ResolvedSource::Segment(segment) => {
                let filter =
                    std::sync::Arc::new(std::sync::Mutex::new(self.filter.as_mut().unwrap()));
                extract_bool_tags_plus_all(
                    &mut block,
                    segment,
                    &self.label,
                    |read| {
                        filter
                            .lock()
                            .unwrap()
                            .containsert(&FragmentEntry(&[read.seq()]))
                    },
                    |reads| {
                        // Virtually combine sequences for filter check
                        let inner: Vec<_> =
                            reads.iter().map(crate::io::WrappedFastQRead::seq).collect();
                        let entry = FragmentEntry(&inner);
                        filter.lock().unwrap().containsert(&entry)
                    },
                );
            }
            ResolvedSource::Tag(tag_name) => {
                let bools = {
                    let result = self.extract_from_tag(&block, &tag_name);
                    result?
                };
                self.store_bool_tag(&mut block, bools);
            }
            ResolvedSource::Name {
                segment,
                split_character,
            } => {
                let bools = self.extract_from_name(&block, segment, split_character);
                self.store_bool_tag(&mut block, bools);
            }
        }
        Ok((block, true))
    }
}

impl Duplicates {
    fn store_bool_tag(&self, block: &mut crate::io::FastQBlocksCombined, bools: Vec<bool>) {
        if block.tags.is_none() {
            block.tags = Some(std::collections::HashMap::new());
        }
        let values = bools.into_iter().map(TagValue::Bool).collect();
        block
            .tags
            .as_mut()
            .expect("Tags should exist after initialization")
            .insert(self.label.clone(), values);
    }

    fn extract_from_tag(
        &mut self,
        block: &crate::io::FastQBlocksCombined,
        tag_name: &str,
    ) -> Result<Vec<bool>> {
        let tags_map = block
            .tags
            .as_ref()
            .with_context(|| format!("Tag '{tag_name}' not found for TagDuplicates source"))?;
        let tag_values = tags_map
            .get(tag_name)
            .with_context(|| format!("Tag '{tag_name}' not found for TagDuplicates source"))?;
        let filter = self.filter.as_mut().expect("init should set filter");
        let mut result = Vec::with_capacity(tag_values.len());
        for value in tag_values {
            let is_duplicate = if let Some(value) = Self::tag_value_to_bytes(value) {
                filter.containsert(&FragmentEntry(&[value.as_slice()]))
            } else {
                false
            };
            result.push(is_duplicate);
        }
        if result.len() != block.len() {
            bail!(
                "Tag '{tag_name}' produced {} entries but block contained {} reads",
                result.len(),
                block.len()
            );
        }
        Ok(result)
    }

    fn extract_from_name(
        &mut self,
        block: &crate::io::FastQBlocksCombined,
        segment: SegmentIndex,
        split_character: u8,
    ) -> Vec<bool> {
        let filter = self.filter.as_mut().expect("init should set filter");
        let segment_block = &block.segments[segment.get_index()];
        let mut result = Vec::with_capacity(segment_block.len());
        for idx in 0..segment_block.len() {
            let read = segment_block.get(idx);
            let name = read.name_without_comment();
            let canonical = read_name_canonical_prefix(name, Some(split_character));
            let owned = canonical.to_vec();
            let is_duplicate = filter.containsert(&FragmentEntry(&[owned.as_slice()]));
            result.push(is_duplicate);
        }
        result
    }

    fn tag_value_to_bytes(value: &TagValue) -> Option<Vec<u8>> {
        match value {
            TagValue::Sequence(hits) => Some(hits.joined_sequence(Some(&[0xff]))),
            TagValue::String(value) => Some(value.to_vec()),
            TagValue::Missing => None,
            _ => {
                dbg!(&value);
                unreachable!()
            }
        }
    }
}
