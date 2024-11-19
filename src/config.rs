use crate::transformations::Transformation;
use anyhow::{bail, Context, Result};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::{collections::HashSet, fmt, marker::PhantomData, process::Output};

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

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConfigInput {
    #[serde(deserialize_with = "string_or_seq_string")]
    pub read1: Vec<String>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub read2: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub index1: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub index2: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
pub enum FileFormat {
    #[serde(alias = "raw")]
    #[serde(alias = "uncompressed")]
    #[serde(alias = "Uncompressed")]
    Raw,
    #[serde(alias = "gzip")]
    #[serde(alias = "gz")]
    #[serde(alias = "Gz")]
    Gzip,
    #[serde(alias = "zstd")]
    #[serde(alias = "zst")]
    #[serde(alias = "Zst")]
    Zstd,
}

impl Default for FileFormat {
    fn default() -> Self {
        FileFormat::Raw
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ConfigOutput {
    pub prefix: String,
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: FileFormat,
    pub compression_level: Option<u8>,
    #[serde(default)]
    pub keep_index: bool,
}

fn default_thread_count() -> usize {
    num_cpus::get()
}

fn default_buffer_size() -> usize {
    100 * 1024 // bytes, per fastq input file
}

fn default_block_size() -> usize {
    //todo: adjust depending on compression mode?
    10000 // in 'molecules', ie. read1, read2, index1, index2 tuples.
}

#[derive(serde::Deserialize, Debug)]
pub struct Options {
    #[serde(default = "default_thread_count")]
    pub thread_count: usize,
    #[serde(default = "default_block_size")]
    pub block_size: usize,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    #[serde(default)]
    pub accept_duplicate_files: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            thread_count: 10,
            block_size: default_block_size(),
            buffer_size: default_buffer_size(),
            accept_duplicate_files: false,
        }
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub input: ConfigInput,
    pub output: Option<ConfigOutput>,
    #[serde(default)]
    pub transform: Vec<Transformation>,
    #[serde(default)]
    pub options: Options,
}

pub fn check_config(config: &Config) -> Result<()> {
    let no_of_files = config.input.read1.len();
    let mut seen = HashSet::new();
    if !config.options.accept_duplicate_files {
        for f in &config.input.read1 {
            if !seen.insert(f) {
                bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
            }
        }
    }

    if let Some(read2) = &config.input.read2 {
        if read2.len() != no_of_files {
            bail!("Number of read2 files must be equal to number of read1 files.");
        }
        if !config.options.accept_duplicate_files {
            for f in read2 {
                if !seen.insert(f) {
                    bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                }
            }
        }
    }
    if let Some(index1) = &config.input.index1 {
        if index1.len() != no_of_files {
            bail!("Number of index1 files must be equal to number of read1 files.");
        }

        if !config.options.accept_duplicate_files {
            for f in index1 {
                if !seen.insert(f) {
                    bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                }
            }
        }
    }
    if let Some(index2) = &config.input.index2 {
        if index2.len() != no_of_files {
            bail!("Number of index2 files must be equal to number of read1 files.");
        }
        if !config.options.accept_duplicate_files {
            for f in index2 {
                if !seen.insert(f) {
                    bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                }
            }
        }
    }

    //no repeated filenames

    for t in &config.transform {
        t.check_config(&config.input)
            .with_context(|| format!("{:?}", t))?;
    }
    Ok(())
}
