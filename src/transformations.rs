use anyhow::{bail, Result};
use serde::{de, Deserialize, Deserializer, Serialize};

use crate::FastQRead;

fn u8_from_string<'de, D>(deserializer: D) -> core::result::Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    // do better hex decoding than this
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
    // do better hex decoding than this
    Ok(s.as_bytes().to_vec())
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
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct ConfigTransformNAndTarget {
    pub n: usize,
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

    pub fn transform(&mut self, block: Vec<crate::Molecule>) -> Vec<crate::Molecule> {
        match self {
            Transformation::Head(config) => block.into_iter().take(config.n).collect(),

            Transformation::Skip(config) => block.into_iter().skip(config.n).collect(),

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
        }
    }
}

fn apply(
    target: Target,
    f: impl Fn(&FastQRead) -> FastQRead,
    block: Vec<crate::Molecule>,
) -> Vec<crate::Molecule> {
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
    }
}
