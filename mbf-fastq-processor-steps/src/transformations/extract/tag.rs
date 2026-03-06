mod duplicates;
mod other_file;

use crate::transformations::{
    FragmentEntry, FragmentEntryForCuckooFilter, OurCuckCooFilter, reproducible_cuckoofilter,
};
pub use duplicates::{Duplicates, PartialDuplicates};
pub use other_file::{OtherFile, PartialOtherFile};
use std::collections::HashSet;
// we settled on the cuckoo filter  after doing experiments/memory_usage_hashset_vs_radis
#[derive(Debug, Clone)]
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
