#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::{fmt, marker::PhantomData, process::Output};

use anyhow::{bail, Context, Result};
use serde::{de, Deserialize, Deserializer, Serialize};

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
struct ConfigInput {
    #[serde(deserialize_with = "string_or_seq_string")]
    read1: Vec<String>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    read2: Option<Vec<String>>,
    index1: Option<Vec<String>>,
    index2: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug)]
enum OutputFormat {
    Raw,
    Gzip,
    Zstd,
}

#[derive(serde::Deserialize, Debug)]
struct ConfigOutput {
    prefix: String,
    suffix: Option<String>,
    format: OutputFormat,
    compression_level: Option<u8>,
    #[serde(default)]
    keep_index: bool,
}

#[derive(serde::Deserialize, Debug)]
struct ConfigTransformHead {
    n: usize,
}

#[derive(serde::Deserialize, Debug)]
#[serde(tag = "action")]
enum ConfigTransformation {
    Head(ConfigTransformHead),
}

#[derive(serde::Deserialize, Debug)]
struct Config {
    input: ConfigInput,
    output: Option<ConfigOutput>,
    transform: Vec<ConfigTransformation>,
}

fn check_config(config: &Config) -> Result<()> {
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

struct InputFiles {
    read1: Vec<std::fs::File>,
    read2: Vec<std::fs::File>,
    index1: Vec<std::fs::File>,
    index2: Vec<std::fs::File>,
}

fn open_files(files: Option<&Vec<String>>) -> Result<Vec<std::fs::File>> {
    match files {
        Some(files) => files
            .iter()
            .map(|f| std::fs::File::open(f).context("Could not open file."))
            .collect(),
        None => Ok(Vec::new()),
    }
}

fn open_input_files(parsed_config: &Config) -> Result<InputFiles> {
    let read1 = open_files(Some(&parsed_config.input.read1))?;
    let read2 = open_files(parsed_config.input.read2.as_ref())?;
    let index1 = open_files(parsed_config.input.index1.as_ref())?;
    let index2 = open_files(parsed_config.input.index2.as_ref())?;
    Ok(InputFiles {
        read1,
        read2,
        index1,
        index2,
    })
}

#[derive(Default)]
struct OutputFiles {
    read1: Option<std::fs::File>,
    read2: Option<std::fs::File>,
    index1: Option<std::fs::File>,
    index2: Option<std::fs::File>,
    reports: Vec<std::fs::File>,
    inspects: Vec<(
        Option<std::fs::File>,
        Option<std::fs::File>,
        Option<std::fs::File>,
        Option<std::fs::File>,
    )>,
}

fn open_output_files(parsed_config: &Config) -> Result<OutputFiles> {
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let suffix =
                output_config
                    .suffix
                    .as_deref()
                    .unwrap_or_else(|| match output_config.format {
                        OutputFormat::Raw => ".fq",
                        OutputFormat::Gzip => ".gz",
                        OutputFormat::Zstd => ".zst",
                    });
            let read1 = Some(std::fs::File::create(format!(
                "{}_1{}",
                output_config.prefix, suffix
            ))?);
            let read2 = match parsed_config.input.read2 {
                Some(_) => Some(std::fs::File::create(format!(
                    "{}_2{}",
                    output_config.prefix, suffix
                ))?),
                None => None,
            };
            let (index1, index2) = if output_config.keep_index {
                (
                    Some(std::fs::File::create(format!(
                        "{}_i1{}",
                        output_config.prefix, suffix
                    ))?),
                    Some(std::fs::File::create(format!(
                        "{}_i2{}",
                        output_config.prefix, suffix
                    ))?),
                )
            } else {
                (None, None)
            };
            let mut reports = Vec::new();
            let mut inspects = Vec::new();
            //todo: open report files.
            OutputFiles {
                read1,
                read2,
                index1,
                index2,
                reports,
                inspects,
            }
        }
        None => OutputFiles::default(),
    })
}

#[derive(Debug)]
struct Read{
    seq: Vec<u8>,
    qual: Vec<u8>,
}


#[derive(Debug)]
struct Molecule {
    read1: Read,
    read2: Option<Read>
    index1: Option<Read>,
    index2: Option<Read>,
    
}

fn main() -> Result<()> {
    //toml from argument
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let raw_config = ex::fs::read_to_string(toml_file).context("Could not read toml file.")?;
    let parsed = toml::from_str::<Config>(&raw_config).context("Could not parse toml file.")?;
    check_config(&parsed)?;
    let input_files = open_input_files(&parsed)?;
    let output_files = open_output_files(&parsed)?;
    dbg!(&parsed);

    ///I need an iterator over all 4 inputs.
    ///before that, I need to decompress...
    ///then I need to do all the 'transformation'
    ///and throw it into the outputs
    ///all of this preferentially streaming, buffered multi threaded...
    Ok(())
}
