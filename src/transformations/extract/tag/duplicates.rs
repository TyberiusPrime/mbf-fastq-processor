#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use std::{collections::HashSet, path::Path};

use super::super::extract_bool_tags_plus_all;

use crate::config::{SegmentIndexOrAll, SegmentOrAll};
use crate::demultiplex::{Demultiplex, DemultiplexInfo};
use crate::transformations::{
    FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter, Step,
    reproducible_cuckoofilter,
};
use serde_valid::Validate;

// we settled on the cucokofilter after doing experiments/memory_usage_hashset_vs_radis
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

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Duplicates {
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,

    pub label: String,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,
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
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
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
        let filter = std::sync::Arc::new(std::sync::Mutex::new(self.filter.as_mut().unwrap()));
        extract_bool_tags_plus_all(
            &mut block,
            self.segment_index.unwrap(),
            &self.label,
            |read| {
                filter
                    .lock()
                    .unwrap()
                    .containsert(&FragmentEntry(&[read.seq()]))
            },
            |reads| {
                // Virtually combine sequences for filter check
                let inner: Vec<_> = reads.iter().map(crate::io::WrappedFastQRead::seq).collect();
                let entry = FragmentEntry(&inner);
                filter.lock().unwrap().containsert(&entry)
            },
        );

        Ok((block, true))
    }
}
