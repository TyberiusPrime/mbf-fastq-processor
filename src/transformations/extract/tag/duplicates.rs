#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use std::{collections::HashSet, path::Path};

use super::super::extract_bool_tags_plus_all;

use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use crate::transformations::{
    reproducible_cuckoofilter, FragmentEntry, FragmentEntryForCuckooFilter, InputInfo,
    OurCuckCooFilter, SegmentOrAll, Step, Transformation,
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
    pub segment: SegmentOrAll,
    pub label: String,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,
    pub seed: u64,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,
}
impl Step for Duplicates {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        self.segment.validate(input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        let filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
            ApproxOrExactFilter::Exact(HashSet::new())
        } else {
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                self.seed,
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
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let filter = std::sync::Arc::new(std::sync::Mutex::new(self.filter.as_mut().unwrap()));
        extract_bool_tags_plus_all(
            &mut block,
            &self.segment,
            &self.label,
            |read| {
                filter
                    .lock()
                    .unwrap()
                    .containsert(&FragmentEntry(&[read.seq()]))
            },
            |reads| {
                // Virtually combine sequences for filter check
                let inner: Vec<_> = reads.iter().map(|x| x.seq()).collect();
                let entry = FragmentEntry(&inner);
                filter.lock().unwrap().containsert(&entry)
            },
        );

        (block, true)
    }
}
