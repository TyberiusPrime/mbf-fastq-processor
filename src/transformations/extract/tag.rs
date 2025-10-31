mod duplicates;
mod other_file_by_name;
mod other_file_by_sequence;

use crate::{
    config::{self, Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll},
    transformations::{
        FragmentEntry, FragmentEntryForCuckooFilter, OurCuckCooFilter, reproducible_cuckoofilter,
    },
};
use anyhow::bail;
pub use duplicates::Duplicates;
pub use other_file_by_name::OtherFileByName;
pub use other_file_by_sequence::OtherFileBySequence;
use serde_valid::Validate;
use std::collections::HashSet;
// we settled on the cuckoo filter  after doing experiments/memory_usage_hashset_vs_radis
#[derive(Debug, Validate, Clone)]
pub enum ApproxOrExactFilter {
    Exact(HashSet<Vec<u8>>),
    Approximate(Box<OurCuckCooFilter<FragmentEntryForCuckooFilter>>),
}

impl ApproxOrExactFilter {
    fn new(false_positive_rate: f64, initial_capacity: usize, seed: u64) -> Self {
        assert!(false_positive_rate >= 0.0);
        if false_positive_rate == 0.0 {
            ApproxOrExactFilter::new_exact()
        } else {
            ApproxOrExactFilter::new_approximate(false_positive_rate, initial_capacity, seed)
        }
    }

    fn new_exact() -> Self {
        ApproxOrExactFilter::Exact(HashSet::new())
    }

    fn new_approximate(false_positive_rate: f64, initial_capacity: usize, seed: u64) -> Self {
        ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
            seed,
            initial_capacity,
            false_positive_rate,
        )))
    }

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

impl ResolvedSource {
    fn parse(source: &str, input_def: &config::Input) -> Result<ResolvedSource, anyhow::Error> {
        let source = source.trim();
        let resolved = if let Some(tag_name) = source.strip_prefix("tag:") {
            let trimmed = tag_name.trim();
            if trimmed.is_empty() {
                bail!("Source tag:<name> may not have an empty name.");
            }
            ResolvedSource::Tag(trimmed.to_string())
        } else if let Some(segment_name) = source.strip_prefix("name:") {
            let trimmed = segment_name.trim();
            if trimmed.is_empty() {
                bail!("TagDuplicates name source requires a segment name");
            }
            let mut segment = Segment(trimmed.to_string());
            let segment_index = segment.validate(input_def)?;
            ResolvedSource::Name {
                segment: segment_index,
                split_character: input_def.options.read_comment_character,
            }
        } else {
            let mut segment = SegmentOrAll(source.to_string());
            ResolvedSource::Segment(segment.validate(input_def)?)
        };
        Ok(resolved)
    }
}
