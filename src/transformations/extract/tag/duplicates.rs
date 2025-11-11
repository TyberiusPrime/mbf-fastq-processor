#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use std::path::Path;

use super::super::extract_bool_tags_plus_all;
use super::{ApproxOrExactFilter, ResolvedSource};
use crate::dna::TagValue;
use crate::transformations::extract::{extract_bool_tags, extract_bool_tags_from_tag};
use crate::transformations::{
    FragmentEntry, InputInfo, read_name_canonical_prefix, tag::DEFAULT_INITIAL_FILTER_CAPACITY,
};
use serde_valid::Validate;

fn default_source() -> String {
    SegmentOrAll::default().0
}

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Duplicates {
    #[serde(default = "default_source")]
    source: String,

    #[serde(default)]
    #[serde(skip)]
    resolved_source: Option<ResolvedSource>,

    pub out_label: String,
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
        self.resolved_source = Some(ResolvedSource::parse(&self.source, input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
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
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let seed = {
            if self.false_positive_rate > 0.0 {
                self.seed
                    .expect("seed should be validated to exist when false_positive_rate > 0.0")
            } else {
                42 // ignored anyway
            }
        };
        self.filter = Some(ApproxOrExactFilter::new(
            self.false_positive_rate,
            DEFAULT_INITIAL_FILTER_CAPACITY,
            seed,
        ));
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let filter = self.filter.as_mut().unwrap();
        match &self.resolved_source.as_ref().unwrap() {
            ResolvedSource::Segment(segment) => {
                let filter = std::cell::RefCell::new(filter);
                extract_bool_tags_plus_all(
                    &mut block,
                    *segment,
                    &self.out_label,
                    |read| {
                        filter
                            .borrow_mut()
                            .containsert(&FragmentEntry(&[read.seq()]))
                    },
                    |reads| {
                        // Virtually combine sequences for filter check
                        let inner: Vec<_> =
                            reads.iter().map(crate::io::WrappedFastQRead::seq).collect();
                        let entry = FragmentEntry(&inner);
                        filter.borrow_mut().containsert(&entry)
                    },
                );
            }
            ResolvedSource::Tag(tag_name) => {
                extract_bool_tags_from_tag(&mut block, &self.out_label, tag_name, |tag_value| {
                    if let Some(value) = Self::tag_value_to_bytes(tag_value) {
                        filter.containsert(&FragmentEntry(&[value.as_slice()]))
                    } else {
                        false
                    }
                });
            }
            ResolvedSource::Name {
                segment,
                split_character,
            } => {
                extract_bool_tags(&mut block, *segment, &self.out_label, |read| {
                    let name = read.name();
                    let canonical = read_name_canonical_prefix(name, Some(*split_character));
                    let owned = canonical.to_vec();
                    filter.containsert(&FragmentEntry(&[owned.as_slice()]))
                });
            }
        }
        Ok((block, true))
    }
}

impl Duplicates {
    fn tag_value_to_bytes(value: &TagValue) -> Option<Vec<u8>> {
        match value {
            TagValue::Location(hits) => Some(hits.joined_sequence(Some(&[0xff]))),
            TagValue::String(value) => Some(value.to_vec()),
            TagValue::Missing => None,
            _ => {
                dbg!(&value);
                unreachable!()
            }
        }
    }
}
