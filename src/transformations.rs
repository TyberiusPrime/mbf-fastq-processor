use std::{
    sync::{Arc, Mutex},
    thread,
};

use anyhow::{bail, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use serde_valid::Validate;

use crate::FastQRead;

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
    )] //we don't check the quality. It's on you if you
    pub readname_end_chars: Vec<u8>,
    #[serde(
        deserialize_with = "u8_from_string",
        default = "default_name_seperator"
    )]
    //we don't check the quality. It's on you if you
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
    pub thread_count: usize,
    #[serde(skip)]
    pub start_time: Option<std::time::Instant>,
    pub n: usize,
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

    ExtractToName(ConfigTransformToName),
    TrimPolyTail(ConfigTransformPolyTail),

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

    pub fn transform(&mut self, mut block: Vec<crate::Molecule>) -> (Vec<crate::Molecule>, bool) {
        match self {
            Transformation::Head(config) => {
                let remaining = config.n - config.so_far;
                if remaining == 0 {
                    return (Vec::new(), false);
                } else {
                    let out: Vec<_> = block.into_iter().take(remaining).collect();
                    let do_continue = remaining > out.len();
                    config.so_far += out.len();
                    (out, do_continue)
                }
            }

            Transformation::Skip(config) => {
                dbg!(&config);
                let remaining = config.n - config.so_far;
                dbg!(remaining);
                if remaining == 0 {
                    (block, true)
                } else {
                    if remaining >= block.len() {
                        config.so_far += block.len();
                        (Vec::new(), true)
                    } else {
                        let out: Vec<_> = block.into_iter().skip(remaining).collect();
                        config.so_far += remaining;
                        (out, true)
                    }
                }
            }

            Transformation::CutStart(config) => {
                apply(config.target, |read| read.cut_start(config.n), block)
            }

            Transformation::CutEnd(config) => {
                apply(config.target, |read| read.cut_end(config.n), block)
            }

            Transformation::MaxLen(config) => {
                apply(config.target, |read| read.max_len(config.n), block)
            }

            Transformation::PreFix(config) => apply(
                config.target,
                |read| read.prefix(&config.seq, &config.qual),
                block,
            ),

            Transformation::PostFix(config) => apply(
                config.target,
                |read| read.postfix(&config.seq, &config.qual),
                block,
            ),

            Transformation::Reverse(config) => apply(config.target, |read| read.reverse(), block),

            Transformation::ExtractToName(config) => {
                block.iter_mut().for_each(|molecule| {
                    let source = match config.source {
                        Target::Read1 => &molecule.read1,
                        Target::Read2 => &molecule
                            .read2
                            .as_ref()
                            .expect("Input def and target mismatch"),
                        Target::Index1 => &molecule
                            .index1
                            .as_ref()
                            .expect("Input def and target mismatch"),
                        Target::Index2 => &molecule
                            .index2
                            .as_ref()
                            .expect("Input def and target mismatch"),
                    };
                    let extracted: Vec<u8> = source
                        .seq
                        .iter()
                        .skip(config.start)
                        .take(config.length)
                        .cloned()
                        .collect();
                    let mut split_pos = None;
                    for letter in config.readname_end_chars.iter() {
                        if let Some(pos) = source.name.iter().position(|&x| x == *letter) {
                            split_pos = Some(pos);
                            break;
                        }
                    }
                    match split_pos {
                        None => {
                            molecule.read1.name.extend(config.separator.iter());
                            molecule.read1.name.extend(extracted.iter());
                        }
                        Some(split_pos) => {
                            let mut new_name = Vec::with_capacity(
                                molecule.read1.name.len()
                                    + config.separator.len()
                                    + extracted.len(),
                            );
                            new_name.extend(molecule.read1.name.iter().take(split_pos));
                            new_name.extend(config.separator.iter());
                            new_name.extend(extracted.iter());
                            new_name.extend(molecule.read1.name.iter().skip(split_pos));
                            molecule.read1.name = new_name;
                        }
                    }
                });
                (block, true)
            }

            Transformation::TrimPolyTail(config) => apply(
                config.target,
                |read| {
                    read.trim_poly_base(config.min_length, config.max_mismatch_rate, 5, config.base)
                },
                block,
            ),

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
                let mut start_local = config.thread_count;
                //now for any multiple of n that's in the range, we print a message
                let offset = counter % config.n;
                for ii in ((counter + offset)..next).step_by(config.n) {
                    start_local += config.n;
                    let rate_total = ii as f64 / config.start_time.unwrap().elapsed().as_secs_f64();
                    let rate_local = start_local as f64 / config.start_time.unwrap().elapsed().as_secs_f64();
                    println!(
                        "Processed Total: {} ({:.2} reads/s), {:.2} reads/s per thread. Elapsed: {}s",
                        ii,
                        rate_total,
                        //start_local,
                        rate_local,
                        config.start_time.unwrap().elapsed().as_secs()
                    );
                }
                config.thread_count += block.len();
                (block, true)
            }
        }
    }
}

fn apply(
    target: Target,
    f: impl Fn(&FastQRead) -> FastQRead,
    block: Vec<crate::Molecule>,
) -> (Vec<crate::Molecule>, bool) {
    (
        match target {
            Target::Read1 => block
                .into_iter()
                .map(|molecule| molecule.replace_read1(f(&molecule.read1)))
                .collect(),
            Target::Read2 => block
                .into_iter()
                .map(|molecule| {
                    let new = f(molecule
                        .read2
                        .as_ref()
                        .expect("Input def and target mismatch"));
                    molecule.replace_read2(Some(new))
                })
                .collect(),
            Target::Index1 => block
                .into_iter()
                .map(|molecule| {
                    let new = f(molecule
                        .index1
                        .as_ref()
                        .expect("Input def and target mismatch"));
                    molecule.replace_index1(Some(new))
                })
                .collect(),
            Target::Index2 => block
                .into_iter()
                .map(|molecule| {
                    let new = f(molecule
                        .index2
                        .as_ref()
                        .expect("Input def and target mismatch"));
                    molecule.replace_index2(Some(new))
                })
                .collect(),
        },
        true,
    )
}
