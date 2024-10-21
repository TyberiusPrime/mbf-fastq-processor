use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{bail, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_valid::Validate;

use crate::{
    io::{self, WrappedFastQReadMut},
    FastQRead,
};

fn u8_from_string<'de, D>(deserializer: D) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.as_bytes().to_vec())
}

fn dna_from_string<'de, D>(deserializer: D) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    //check whether it's DNA bases...
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N') {
            return Err(serde::de::Error::custom(format!("Invalid DNA base: {}", c)));
        }
    }
    Ok(s.as_bytes().to_vec())
}

fn base_or_dot<'de, D>(deserializer: D) -> core::result::Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.to_uppercase();
    if s.len() != 1 {
        return Err(serde::de::Error::custom(format!(
            "Single DNA base or '.' only): was '{}'",
            s
        )));
    }
    for c in s.chars() {
        if !matches!(c, 'A' | 'C' | 'G' | 'T' | 'N' | '.') {
            return Err(serde::de::Error::custom(format!(
                "Invalid DNA base (. for any also allowed): {}",
                c
            )));
        }
    }
    let out = s.as_bytes()[0];
    Ok(out)
}

pub fn u8_from_char_or_number<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = u8;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("either a character or a number")
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v as u8)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v.len() {
                0 => Err(E::custom("empty string")),
                1 => Ok(v.bytes().next().unwrap() as u8),
                _ => Err(E::custom("string should be exactly one character long")),
            }
        }
    }

    deserializer.deserialize_any(Visitor)
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub enum Target {
    Read1,
    Read2,
    Index1,
    Index2,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformN {
    pub n: usize,
    #[serde(skip)]
    pub so_far: usize,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformNAndTarget {
    pub n: usize,
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformTarget {
    pub target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformText {
    pub target: Target,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: Vec<u8>,

    #[serde(deserialize_with = "u8_from_string")] //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: Vec<u8>,
}

fn default_readname_end_chars() -> Vec<u8> {
    vec![b' ', b'/']
}

fn default_name_seperator() -> Vec<u8> {
    vec![b'_']
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformToName {
    pub source: Target,
    pub start: usize,
    pub length: usize,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_readname_end_chars"
    )]
    pub readname_end_chars: Vec<u8>,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_seperator"
    )]
    pub separator: Vec<u8>,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformPolyTail {
    pub target: Target,
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 10.)]
    pub max_mismatch_rate: f32,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformProgress {
    #[serde(skip)]
    pub total_count: Arc<Mutex<usize>>,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    pub n: usize,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformQual {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min: u8,
}
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformQualFloat {
    pub target: Target,
    pub min: f32,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
pub struct ConfigTransformQualifiedBases {
    #[serde(deserialize_with = "u8_from_char_or_number")]
    min_quality: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    min_percentage: f32,
    target: Target,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformFilterTooManyN {
    target: Target,
    max: usize, 
}

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(tag = "action")]
pub enum Transformation {
    Head(ConfigTransformN),
    Skip(ConfigTransformN),

    CutStart(ConfigTransformNAndTarget),
    CutEnd(ConfigTransformNAndTarget),
    MaxLen(ConfigTransformNAndTarget),

    PreFix(ConfigTransformText),
    PostFix(ConfigTransformText),

    Reverse(ConfigTransformTarget),
    ConvertPhred64To33,
    ExtractToName(ConfigTransformToName),

    TrimPolyTail(ConfigTransformPolyTail),
    TrimQualityStart(ConfigTransformQual),
    TrimQualityEnd(ConfigTransformQual),

    FilterMinLen(ConfigTransformNAndTarget),
    FilterMeanQuality(ConfigTransformQualFloat),
    FilterQualifiedBases(ConfigTransformQualifiedBases),
    FilterTooManyN(ConfigTransformFilterTooManyN),

    Progress(ConfigTransformProgress),
}

fn verify_target(target: Target, input_def: &crate::config::ConfigInput) -> Result<()> {
    match target {
        Target::Read1 => {}
        Target::Read2 => {
            if input_def.read2.is_none() {
                bail!("Read2 is not defined in the input section, but used by transformation");
            }
        }
        Target::Index1 => {
            if input_def.index1.is_none() {
                bail!("Index1 is not defined in the input section, but used by transformation");
            }
        }
        Target::Index2 => {
            if input_def.index2.is_none() {
                bail!("Index2 is not defined in the input section, but used by transformation");
            }
        }
    }
    Ok(())
}

impl Transformation {
    pub fn needs_serial(&self) -> bool {
        // ie. must see all the reads.
        match self {
            Transformation::Head(_) => true,
            Transformation::Skip(_) => true,
            _ => false,
        }
    }

    pub fn check_config(&self, input_def: &crate::config::ConfigInput) -> Result<()> {
        match self {
            Transformation::CutStart(c) | Transformation::CutEnd(c) => {
                return verify_target(c.target, input_def)
            }
            Transformation::PreFix(c) | Transformation::PostFix(c) => {
                verify_target(c.target, input_def)?;
                if c.seq.len() != c.qual.len() {
                    bail!("Seq and qual must be the same length");
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn transform(
        &mut self,
        mut block: io::FastQBlocksCombined,
    ) -> (io::FastQBlocksCombined, bool) {
        match self {
            Transformation::Head(config) => {
                let remaining = config.n - config.so_far;
                if remaining == 0 {
                    return (block.empty(), false);
                } else {
                    block.resize(remaining.min(block.len()));
                    let do_continue = remaining > block.len();
                    config.so_far += block.len();
                    (block, do_continue)
                }
            }

            Transformation::Skip(config) => {
                let remaining = config.n - config.so_far;
                if remaining == 0 {
                    (block, true)
                } else {
                    if remaining >= block.len() {
                        config.so_far += block.len();
                        (block.empty(), true)
                    } else {
                        let here = remaining.min(block.len());
                        config.so_far += here;
                        block.block_read1.entries.drain(0..here);
                        if let Some(ref mut read2) = block.block_read2 {
                            read2.entries.drain(0..here);
                            assert_eq!(read2.len(), block.block_read1.len());
                        }
                        if let Some(ref mut index1) = block.block_index1 {
                            index1.entries.drain(0..here);
                        }
                        if let Some(ref mut index2) = block.block_index2 {
                            index2.entries.drain(0..here);
                        }
                        (block, true)
                    }
                }
            }

            Transformation::CutStart(config) => {
                apply_in_place(config.target, |read| read.cut_start(config.n), &mut block);
                (block, true)
            }

            Transformation::CutEnd(config) => {
                apply_in_place(config.target, |read| read.cut_end(config.n), &mut block);
                (block, true)
            }

            Transformation::MaxLen(config) => {
                apply_in_place(config.target, |read| read.max_len(config.n), &mut block);
                (block, true)
            }

            Transformation::PreFix(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.prefix(&config.seq, &config.qual),
                    &mut block,
                );
                (block, true)
            }

            Transformation::PostFix(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.postfix(&config.seq, &config.qual),
                    &mut block,
                );
                (block, true)
            }

            Transformation::Reverse(config) => {
                apply_in_place_wrapped(config.target, |read| read.reverse(), &mut block);
                (block, true)
            }

            Transformation::ConvertPhred64To33 => {
                block.apply_mut(|read1, read2, index1, index2| {
                    let qual = read1.qual();
                    let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                    if new_qual.iter().any(|x| *x < 33) {
                        panic!("Phred 64-33 conversion yielded values below 33 -> wasn't Phred 64 to begin with");
                    }
                    read1.replace_qual(new_qual);
                    if let Some(inner_read2) = read2 {
                        let qual = inner_read2.qual();
                        let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                        inner_read2.replace_qual(new_qual);
                    }
                    if let Some(index1) = index1 {
                        let qual = index1.qual();
                        let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                        index1.replace_qual(new_qual);
                    }
                    if let Some(index2) = index2 {
                        let qual = index2.qual();
                        let new_qual: Vec<_> = qual.iter().map(|x| x.saturating_sub(31)).collect();
                        index2.replace_qual(new_qual);
                    }
                });
                (block, true)
            }

            Transformation::ExtractToName(config) => {
                block.apply_mut(|read1, read2, index1, index2| {
                    let source = match config.source {
                        Target::Read1 => &read1,
                        Target::Read2 => &read2.as_ref().expect("Input def and target mismatch"),
                        Target::Index1 => &index1.as_ref().expect("Input def and target mismatch"),
                        Target::Index2 => &index2.as_ref().expect("Input def and target mismatch"),
                    };
                    let extracted: Vec<u8> = source
                        .seq()
                        .iter()
                        .skip(config.start)
                        .take(config.length)
                        .cloned()
                        .collect();

                    let name = read1.name();
                    let mut split_pos = None;
                    for letter in config.readname_end_chars.iter() {
                        if let Some(pos) = name.iter().position(|&x| x == *letter) {
                            split_pos = Some(pos);
                            break;
                        }
                    }
                    let new_name = match split_pos {
                        None => {
                            let mut new_name: Vec<u8> = name.into();
                            new_name.extend(config.separator.iter());
                            new_name.extend(extracted.iter());
                            new_name
                        }
                        Some(split_pos) => {
                            let mut new_name = Vec::with_capacity(
                                name.len() + config.separator.len() + extracted.len(),
                            );
                            new_name.extend(name.iter().take(split_pos));
                            new_name.extend(config.separator.iter());
                            new_name.extend(extracted.iter());
                            new_name.extend(name.iter().skip(split_pos));
                            new_name
                        }
                    };
                    read1.replace_name(new_name);
                });
                (block, true)
            }

            Transformation::TrimPolyTail(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| {
                        read.trim_poly_base(
                            config.min_length,
                            config.max_mismatch_rate,
                            5,
                            config.base,
                        )
                    },
                    &mut block,
                );
                (block, true)
            }

            Transformation::TrimQualityStart(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.trim_quality_start(config.min),
                    &mut block,
                );
                (block, true)
            }
            Transformation::TrimQualityEnd(config) => {
                apply_in_place_wrapped(
                    config.target,
                    |read| read.trim_quality_end(config.min),
                    &mut block,
                );
                (block, true)
            }
            Transformation::FilterMinLen(config) => {
                apply_filter(config.target, &mut block, |read| {
                    read.seq().len() >= config.n
                });
                (block, true)
            }

            Transformation::FilterMeanQuality(config) => {
                apply_filter(config.target, &mut block, |read| {
                    let qual = read.qual();
                    let sum: usize = qual.iter().map(|x| *x as usize).sum();
                    let avg_qual = sum as f32 / qual.len() as f32;
                    avg_qual >= config.min
                });
                (block, true)
            }

            Transformation::FilterQualifiedBases(config) => 
            {
                apply_filter(config.target, &mut block, |read| {
                    let qual = read.qual();
                    let sum: usize = qual.iter().map(|x| (*x >= config.min_quality) as usize).sum();
                    let pct = sum as f32 / qual.len() as f32;
                    pct >= config.min_percentage
                });
                (block, true)
            }
          Transformation::FilterTooManyN(config) => 
            {
                apply_filter(config.target, &mut block, |read| {
                    let seq = read.seq();
                    let sum: usize = seq.iter().map(|x| (*x == b'N') as usize).sum();
                    sum <= config.max
                });
                (block, true)
            }

            Transformation::Progress(config) => {
                if let None = config.start_time {
                    config.start_time = Some(std::time::Instant::now());
                }
                let (counter, next) = {
                    let mut counter = config.total_count.lock().unwrap();
                    //    println!("Thread {:?}", thread::current().id());
                    let val = *counter;
                    let next = *counter + block.len();
                    *counter = next;
                    drop(counter);
                    (val, next)
                };
                //now for any multiple of n that's in the range, we print a message
                let offset = counter % config.n;
                for ii in ((counter + offset)..next).step_by(config.n) {
                    let elapsed = config.start_time.unwrap().elapsed().as_secs_f64();
                    let rate_total = ii as f64 / elapsed;
                    if elapsed > 1.0 {
                        println!(
                            "Processed Total: {} ({:.2} molecules/s), Elapsed: {}s",
                            ii,
                            rate_total,
                            config.start_time.unwrap().elapsed().as_secs()
                        );
                    } else {
                        println!(
                            "Processed Total: {}, Elapsed: {}s",
                            ii,
                            config.start_time.unwrap().elapsed().as_secs()
                        );
                    }
                }
                (block, true)
            }
            _ => {
                todo!()
            }
        }
    }
}

/// for the cases where the actual data is irrelevant.
fn apply_in_place(
    target: Target,
    f: impl Fn(&mut io::FastQRead),
    block: &mut io::FastQBlocksCombined,
) {
    match target {
        Target::Read1 => {
            for read in block.block_read1.entries.iter_mut() {
                f(read);
            }
        }
        Target::Read2 => {
            for read in block.block_read2.as_mut().unwrap().entries.iter_mut() {
                f(read);
            }
        }
        Target::Index1 => {
            for read in block.block_index1.as_mut().unwrap().entries.iter_mut() {
                f(read);
            }
        }
        Target::Index2 => {
            for read in block.block_index2.as_mut().unwrap().entries.iter_mut() {
                f(read);
            }
        }
    }
}

/// for the cases where the actual data is relevant.
fn apply_in_place_wrapped(
    target: Target,
    f: impl Fn(&mut io::WrappedFastQReadMut),
    block: &mut io::FastQBlocksCombined,
) {
    match target {
        Target::Read1 => block.block_read1.apply_mut(f),
        Target::Read2 => block
            .block_read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index1 => block
            .block_index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
        Target::Index2 => block
            .block_index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut(f),
    }
}

fn apply_filter(
    target: Target,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQRead) -> bool,
) {
    let target = match target {
        Target::Read1 => &block.block_read1,
        Target::Read2 => &block.block_read2.as_ref().unwrap(),
        Target::Index1 => &block.block_index1.as_ref().unwrap(),
        Target::Index2 => &block.block_index2.as_ref().unwrap(),
    };
    let keep: Vec<_> = target.apply(f);
    let mut iter = keep.iter();
    block.block_read1.entries.retain(|_| *iter.next().unwrap());
    if let Some(ref mut read2) = block.block_read2 {
        let mut iter = keep.iter();
        read2.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index1) = block.block_index1 {
        let mut iter = keep.iter();
        index1.entries.retain(|_| *iter.next().unwrap());
    }
    if let Some(ref mut index2) = block.block_index2 {
        let mut iter = keep.iter();
        index2.entries.retain(|_| *iter.next().unwrap());
    }
}
