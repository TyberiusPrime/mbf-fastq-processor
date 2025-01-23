use crate::transformations::Transformation;
use anyhow::{bail, Context, Result};
use serde_valid::Validate;
use std::collections::HashSet;

pub mod deser;

use deser::{string_or_seq_string, string_or_seq_string_or_none};

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(deserialize_with = "string_or_seq_string")]
    pub read1: Vec<String>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub read2: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub index1: Option<Vec<String>>,
    #[serde(default, deserialize_with = "string_or_seq_string_or_none")]
    pub index2: Option<Vec<String>>,
    #[serde(default)]
    pub interleaved: bool,
}

#[derive(serde::Deserialize, Debug, Copy, Clone, Default)]
pub enum FileFormat {
    #[serde(alias = "raw")]
    #[serde(alias = "uncompressed")]
    #[serde(alias = "Uncompressed")]
    #[default]
    Raw,
    #[serde(alias = "gzip")]
    #[serde(alias = "gz")]
    #[serde(alias = "Gz")]
    Gzip,
    #[serde(alias = "zstd")]
    #[serde(alias = "zst")]
    #[serde(alias = "Zst")]
    Zstd,
    #[serde(alias = "none")] // we need this so you can disable the output, but set a prefix for
    // the Reports
    None,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(serde::Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub prefix: String,
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: FileFormat,
    pub compression_level: Option<u8>,

    #[serde(default)]
    pub stdout: bool,
    #[serde(default)]
    pub interleave: bool,
    #[serde(default)]
    pub keep_index: bool,
    #[serde(default)]
    pub output_hash: bool,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub enum Target {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
}

#[derive(serde::Deserialize, Debug, Copy, Clone)]
pub enum TargetPlusAll {
    #[serde(alias = "read1")]
    Read1,
    #[serde(alias = "read2")]
    Read2,
    #[serde(alias = "index1")]
    Index1,
    #[serde(alias = "index2")]
    Index2,
    #[serde(alias = "all")]
    All,
}

#[derive(serde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
    pub source: Target,
    pub start: usize,
    #[validate(minimum = 1)]
    pub length: usize,
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
#[serde(deny_unknown_fields)]
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
    pub input: Input,
    pub output: Option<Output>,
    #[serde(default)]
    pub transform: Vec<Transformation>,
    #[serde(default)]
    pub options: Options,
}

impl Config {
    pub fn check(&mut self) -> Result<()> {
        let no_of_files = self.input.read1.len();
        if no_of_files == 0 {
            bail!("No read1 files specified / empty list.");
        }
        let mut seen = HashSet::new();
        if !self.options.accept_duplicate_files {
            for f in &self.input.read1 {
                if !seen.insert(f) {
                    bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                }
            }
        }

        if let Some(read2) = &self.input.read2 {
            if self.input.interleaved {
                bail!("If interleaved is set, read2 must not be set");
            }
            if read2.len() != no_of_files {
                bail!("Number of read2 files must be equal to number of read1 files.");
            }
            if !self.options.accept_duplicate_files {
                for f in read2 {
                    if !seen.insert(f) {
                        bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                    }
                }
            }
        } else if let Some(output) = &self.output {
            if output.interleave {
                bail!("Interleaving requires read2 files to be specified.");
            }
        }

        if let Some(index1) = &self.input.index1 {
            if index1.len() != no_of_files {
                bail!("Number of index1 files must be equal to number of read1 files.");
            }

            if !self.options.accept_duplicate_files {
                for f in index1 {
                    if !seen.insert(f) {
                        bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                    }
                }
            }
        }
        if let Some(index2) = &self.input.index2 {
            if self.input.index1.is_none() {
                bail!("index2 file(s) set without index1 file(s) present. Start with index1")
            }
            if index2.len() != no_of_files {
                bail!("Number of index2 files must be equal to number of read1 files.");
            }
            if !self.options.accept_duplicate_files {
                for f in index2 {
                    if !seen.insert(f) {
                        bail!("Repeated filename: {}. Probably not what you want. Set options.accept_duplicate_files = true to ignore.", f);
                    }
                }
            }
        }

        //no repeated filenames
        for t in &self.transform {
            t.check_config(&self.input, &self.output, &self.transform)
                .with_context(|| format!("{t:?}"))?;
        }

        //apply output if set
        if let Some(output) = &mut self.output {
            if output.stdout {
                output.format = FileFormat::Raw;
                output.interleave = self.input.read2.is_some();
            }
            if output.keep_index {
                if self.input.index1.is_none() {
                    bail!("keep_index is set, but no index1 files are specified.");
                }
                if self.input.index2.is_none() {
                    bail!("keep_index is set, but no index2 files are specified.");
                }
            }
        }

        Ok(())
    }
}
