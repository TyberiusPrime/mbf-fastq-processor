use anyhow::Result;
use std::{collections::HashSet, path::Path};

use super::{
    apply_filter, apply_filter_all, extend_seed, reproducible_cuckoofilter, validate_target,
    FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, KeepOrRemove, OurCuckCooFilter, Step,
    Target, TargetPlusAll, Transformation,
};
use crate::{
    config::deser::{option_u8_from_string, u8_from_char_or_number},
    demultiplex::{DemultiplexInfo, Demultiplexed},
};
use serde_valid::Validate;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

impl Step for Head {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            (block.empty(), false)
        } else {
            block.resize(remaining.min(block.len()));
            let do_continue = remaining > block.len();
            self.so_far += block.len();
            (block, do_continue)
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Skip {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

impl Step for Skip {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let remaining = self.n - self.so_far;
        if remaining == 0 {
            (block, true)
        } else if remaining >= block.len() {
            self.so_far += block.len();
            (block.empty(), true)
        } else {
            let here = remaining.min(block.len());
            self.so_far += here;
            block.drain(0..here);
            (block, true)
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Empty {
    pub target: Target,
}

impl Step for Empty {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| !read.seq().is_empty());
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MinLen {
    pub n: usize,
    pub target: Target,
}

impl Step for MinLen {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| read.seq().len() >= self.n);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MaxLen {
    pub n: usize,
    pub target: Target,
}

impl Step for MaxLen {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| read.seq().len() <= self.n);
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MeanQuality {
    pub target: Target,
    pub min: f32,
}
impl Step for MeanQuality {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| {
            let qual = read.qual();
            let sum: usize = qual.iter().map(|x| *x as usize).sum();
            let avg_qual = sum as f32 / qual.len() as f32;
            avg_qual >= self.min
        });
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct QualifiedBases {
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub min_percentage: f32,
    pub target: Target,
}

impl Step for QualifiedBases {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| {
            let qual = read.qual();
            let sum: usize = qual
                .iter()
                .map(|x| usize::from(*x >= self.min_quality))
                .sum();
            let pct = sum as f32 / qual.len() as f32;
            pct >= self.min_percentage
        });
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TooManyN {
    pub target: Target,
    pub n: usize,
}
impl Step for TooManyN {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| {
            let seq = read.seq();
            let sum: usize = seq.iter().map(|x| usize::from(*x == b'N')).sum();
            sum <= self.n
        });
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct LowComplexity {
    pub target: Target,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub threshold: f32,
}

impl Step for LowComplexity {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(self.target, &mut block, |read| {
            // Calculate the number of transitions
            let mut transitions = 0;
            let seq = read.seq();
            for ii in 0..seq.len() - 1 {
                if seq[ii] != seq[ii + 1] {
                    transitions += 1;
                }
            }
            let ratio = transitions as f32 / (read.len() - 1) as f32;
            ratio >= self.threshold
        });
        (block, true)
    }
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub p: f32,
    pub seed: u64,
}

impl Step for Sample {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        use rand_chacha::rand_core::SeedableRng;
        let extended_seed = extend_seed(self.seed);

        // Singlecore approach to avoid reinitializing RNG
        let mut rng = rand_chacha::ChaChaRng::from_seed(extended_seed);
        apply_filter(Target::Read1, &mut block, |_| {
            use rand::Rng;
            rng.random_bool(f64::from(self.p))
        });
        (block, true)
    }
}

// we settled on the cuckofilter after doing experiments/memory_usage_hashset_vs_radis
#[derive(Debug, Validate, Clone)]
pub enum ApproxOrExactFilter {
    Exact(HashSet<Vec<u8>>),
    Approximate(Box<OurCuckCooFilter<FragmentEntryForCuckooFilter>>),
}

impl ApproxOrExactFilter {
    fn contains(&self, seq: &FragmentEntry) -> bool {
        match self {
            ApproxOrExactFilter::Exact(hashset) => hashset.contains(&seq.to_continuous_vec()),
            ApproxOrExactFilter::Approximate(filter) => filter.contains(seq),
        }
    }

    fn containsert(&mut self, seq: &FragmentEntry) -> bool {
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

    fn insert(&mut self, seq: &FragmentEntry) {
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

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Duplicates {
    pub target: TargetPlusAll,
    #[serde(default)]
    pub invert: bool,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,
    pub seed: u64,
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,
}
impl Step for Duplicates {
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
        let filter = self.filter.as_mut().unwrap();
        //target is a Target, and TargetPulsAll
        if let Ok(target) = self.target.try_into() {
            apply_filter(target, &mut block, |read| {
                if filter.containsert(&FragmentEntry(read.seq(), None, None, None)) {
                    self.invert
                } else {
                    !self.invert
                }
            });
        } else {
            apply_filter_all(&mut block, |read1, read2, index1, index2| {
                // Virtually combine sequences for filter check
                let seq = FragmentEntry(
                    read1.seq(),
                    read2.as_ref().map(|r| r.seq()),
                    index1.as_ref().map(|r| r.seq()),
                    index2.as_ref().map(|r| r.seq()),
                );
                if filter.containsert(&seq) {
                    self.invert
                } else {
                    !self.invert
                }
            });
        }
        (block, true)
    }
}
#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct OtherFile {
    pub keep_or_remove: KeepOrRemove,
    pub filename: String,
    pub seed: u64,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,

    #[serde(deserialize_with = "option_u8_from_string")]
    #[serde(default)]
    pub readname_end_chars: Option<Vec<u8>>,
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,
}
impl Step for OtherFile {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        let mut filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
            ApproxOrExactFilter::Exact(HashSet::new())
        } else {
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                self.seed,
                100_000,
                self.false_positive_rate,
            )))
        };
        crate::io::apply_to_readnames(&self.filename, &mut |read_name| {
            filter.insert(&FragmentEntry(read_name, None, None, None));
        })?;
        self.filter = Some(filter);
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_filter(Target::Read1, &mut block, |read| {
            let filter = self.filter.as_ref().unwrap();
            let query = match &self.readname_end_chars {
                None => read.name(),
                Some(split_chars) => {
                    let mut split_pos = None;
                    let name = read.name();
                    for letter in split_chars {
                        if let Some(pos) = name.iter().position(|&x| x == *letter) {
                            split_pos = Some(pos);
                            break;
                        }
                    }
                    match split_pos {
                        None => name,
                        Some(split_pos) => &name[..split_pos],
                    }
                }
            };

            let mut keep = filter.contains(&FragmentEntry(query, None, None, None));
            if let KeepOrRemove::Remove = self.keep_or_remove {
                keep = !keep;
            }
            keep
        });
        (block, true)
    }
}
