use anyhow::Result;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

use super::{
    apply_filter, apply_filter_all, extend_seed, reproducible_cuckoofilter,
    ConfigTransformNAndTarget, KeepOrRemove, OurCuckCooFilter, Target,
    TargetPlusAll,
};
use crate::config::deser::{option_u8_from_string, u8_from_char_or_number};
use serde_valid::Validate;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformN {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformQualFloat {
    pub target: Target,
    pub min: f32,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformQualifiedBases {
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub min_percentage: f32,
    pub target: Target,
}

//todo: unify with ConfigTransformN
#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformFilterTooManyN {
    pub target: Target,
    pub n: usize,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformFilterLowComplexity {
    pub target: Target,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub threshold: f32,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformFilterSample {
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub p: f32,
    pub seed: u64,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformFilterDuplicates {
    pub target: TargetPlusAll,
    #[serde(default)]
    pub invert: bool,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,
    pub seed: u64,
    #[serde(skip)]
    pub filter: Option<OurCuckCooFilter>,
}

// we settled on the cuckofilter after doing experiments/memory_usage_hashset_vs_radis
#[derive(Debug, Validate, Clone)]
pub enum ReadNameFilter {
    Exact(HashSet<Vec<u8>>),
    Approximate(Box<OurCuckCooFilter>),
}

#[derive(serde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigTransformFilterOtherFile {
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
    pub filter: Option<ReadNameFilter>,
}

pub fn transform_head(
    config: &mut ConfigTransformN,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let remaining = config.n - config.so_far;
    if remaining == 0 {
        (block.empty(), false)
    } else {
        block.resize(remaining.min(block.len()));
        let do_continue = remaining > block.len();
        config.so_far += block.len();
        (block, do_continue)
    }
}

pub fn transform_skip(
    config: &mut ConfigTransformN,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    {
        let remaining = config.n - config.so_far;
        if remaining == 0 {
            (block, true)
        } else if remaining >= block.len() {
            config.so_far += block.len();
            (block.empty(), true)
        } else {
            let here = remaining.min(block.len());
            config.so_far += here;
            block.drain(0..here);
            (block, true)
        }
    }
}

pub fn transform_filter_min_len(
    config: &mut ConfigTransformNAndTarget,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        read.seq().len() >= config.n
    });
    (block, true)
}

pub fn transform_filter_max_len(
    config: &mut ConfigTransformNAndTarget,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        read.seq().len() <= config.n
    });
    (block, true)
}

#[allow(clippy::cast_precision_loss)]
pub fn transform_filter_mean_quality(
    config: &mut ConfigTransformQualFloat,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        let qual = read.qual();
        let sum: usize = qual.iter().map(|x| *x as usize).sum();
        let avg_qual = sum as f32 / qual.len() as f32;
        avg_qual >= config.min
    });
    (block, true)
}

#[allow(clippy::cast_precision_loss)]
pub fn transform_filter_qualified_bases(
    config: &mut ConfigTransformQualifiedBases,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        let qual = read.qual();
        let sum: usize = qual
            .iter()
            .map(|x| usize::from(*x >= config.min_quality))
            .sum();
        let pct = sum as f32 / qual.len() as f32;
        pct >= config.min_percentage
    });
    (block, true)
}

pub fn transform_filter_too_many_n(
    config: &mut ConfigTransformFilterTooManyN,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        let seq = read.seq();
        let sum: usize = seq.iter().map(|x| usize::from(*x == b'N')).sum();
        sum <= config.n
    });
    (block, true)
}

pub fn transform_filter_sample(
    config: &mut ConfigTransformFilterSample,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    let extended_seed = extend_seed(config.seed);

    //todo: I think we should singlecore this, and have just one rng in total,
    //not reinitalizie it over and over
    let mut rng = rand_chacha::ChaChaRng::from_seed(extended_seed);
    apply_filter(Target::Read1, &mut block, |_| {
        rng.gen_bool(f64::from(config.p))
    });
    (block, true)
}

pub fn transform_filter_duplicates(
    config: &mut ConfigTransformFilterDuplicates,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    if config.filter.is_none() {
        config.filter = Some(reproducible_cuckoofilter(
            config.seed,
            1_000_000,
            config.false_positive_rate,
        ));
    }
    let filter = config.filter.as_mut().unwrap();
    if let Ok(target) = config.target.try_into() {
        apply_filter(target, &mut block, |read| {
            if filter.contains(read.seq()) {
                config.invert
            } else {
                filter.insert(read.seq());
                !config.invert
            }
        });
    } else {
        apply_filter_all(&mut block, |read1, read2, index1, index2| {
            //wish I could feed tehse into the filter without creating the vec
            let mut seq: Vec<_> = Vec::new();
            seq.extend(read1.seq().iter());
            if let Some(read2) = read2 {
                seq.extend(read2.seq().iter());
            }
            if let Some(index1) = index1 {
                seq.extend(index1.seq().iter());
            }
            if let Some(index2) = index2 {
                seq.extend(index2.seq().iter());
            }

            if filter.contains(&seq) {
                config.invert
            } else {
                filter.insert(&seq);
                !config.invert
            }
        });
    }
    (block, true)
}

#[allow(clippy::cast_precision_loss)]
pub fn transform_filter_low_complexity(
    config: &mut ConfigTransformFilterLowComplexity,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(config.target, &mut block, |read| {
        //how many transitions are there in read.seq()
        let mut transitions = 0;
        let seq = read.seq();
        for ii in 0..seq.len() - 1 {
            if seq[ii] != seq[ii + 1] {
                transitions += 1;
            }
        }
        let ratio = transitions as f32 / (read.len() - 1) as f32;
        ratio >= config.threshold
    });
    (block, true)
}

pub fn init_filter_other_file(config: &mut ConfigTransformFilterOtherFile) -> Result<()> {
    let mut filter: ReadNameFilter = if config.false_positive_rate == 0.0 {
        ReadNameFilter::Exact(HashSet::new())
    } else {
        ReadNameFilter::Approximate(Box::new(reproducible_cuckoofilter(
            config.seed,
            100_000,
            config.false_positive_rate,
        )))
    };
    crate::io::apply_to_readnames(&config.filename, &mut |read_name| match &mut filter {
        ReadNameFilter::Exact(set) => {
            set.insert(read_name.to_vec());
        }
        ReadNameFilter::Approximate(filter) => {
            filter.insert(read_name);
        }
    })?;
    config.filter = Some(filter);
    Ok(())
}

pub fn transform_filter_other_file(
    config: &mut ConfigTransformFilterOtherFile,
    mut block: crate::io::FastQBlocksCombined,
) -> (crate::io::FastQBlocksCombined, bool) {
    apply_filter(Target::Read1, &mut block, |read| {
        let filter = config.filter.as_ref().unwrap();
        let query = match &config.readname_end_chars {
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

        let mut keep = match filter {
            ReadNameFilter::Exact(set) => set.contains(query),
            ReadNameFilter::Approximate(filter) => filter.contains(query),
        };
        if let KeepOrRemove::Remove = config.keep_or_remove {
            keep = !keep;
        }
        keep
    });
    (block, true)
}
