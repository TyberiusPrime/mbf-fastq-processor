use anyhow::{bail, Context, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{fmt, marker::PhantomData, process::Output};

fn string_or_seq_string<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(vec![value.to_owned()])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Deserialize::deserialize(de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}
fn string_or_seq_string_or_none<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrVec(PhantomData<Option<Vec<String>>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Option<Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![value.to_owned()]))
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: de::SeqAccess<'de>,
        {
            Ok(Some(Deserialize::deserialize(
                de::value::SeqAccessDeserializer::new(visitor),
            )?))
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfigInput {
    #[serde(deserialize_with = "string_or_seq_string")]
    pub read1: Vec<String>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub read2: Option<Vec<String>>,
    pub index1: Option<Vec<String>>,
    pub index2: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
pub enum OutputFormat {
    Raw,
    Gzip,
    Zstd,
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfigOutput {
    pub prefix: String,
    pub suffix: Option<String>,
    pub format: OutputFormat,
    pub compression_level: Option<u8>,
    #[serde(default)]
    pub keep_index: bool,
}

#[derive(serde::Deserialize, Debug)]
pub struct ConfigTransformHead {
    pub n: usize,
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ConfigTransformation {
    Head(ConfigTransformHead),
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    pub input: ConfigInput,
    pub output: Option<ConfigOutput>,
    pub transform: Vec<ConfigTransformation>,
}

pub fn check_config(config: &Config) -> Result<()> {
    let no_of_files = config.input.read1.len();
    if let Some(read2) = &config.input.read2 {
        if read2.len() != no_of_files {
            bail!("Number of read2 files must be equal to number of read1 files.");
        }
    }
    if let Some(index1) = &config.input.index1 {
        if index1.len() != no_of_files {
            bail!("Number of index1 files must be equal to number of read1 files.");
        }
    }
    if let Some(index2) = &config.input.index2 {
        if index2.len() != no_of_files {
            bail!("Number of index2 files must be equal to number of read1 files.");
        }
    }
    Ok(())
}
