use anyhow::{Context, Result};
use noodles::bam;
use std::{ops::Range, path::Path};

pub mod fileformats;
pub mod input;
pub mod output;
pub mod parsers;
pub mod reads;

pub use input::{
    detect_input_format, open_file, open_input_files, DetectedInputFormat, InputFile, InputFiles,
};
use parsers::Parser;
pub use reads::{
    longest_suffix_that_is_a_prefix, FastQBlock, FastQBlocksCombined, FastQElement, FastQRead,
    Position, SegmentsCombined, WrappedFastQRead, WrappedFastQReadMut,
};

pub use output::{write_read_to_bam, BamOutput};
pub use output::compressed_output;

/// Given a fastq or bam file, run a call back on all read names
pub fn apply_to_read_names(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    let filename = filename.as_ref();
    let ext = filename
        .extension()
        .context("Could not detect filetype from extension")?
        .to_string_lossy();
    if ext == "sam" || ext == "bam" {
        {
            let ignore_unmapped =
                ignore_unmapped.expect("When using bam/sam ignore_unmapped must be set.");
            let mut reader = bam::io::reader::Builder.build_from_path(filename)?;
            reader.read_header()?;
            for result in reader.records() {
                let record = result?;
                if ignore_unmapped && record.reference_sequence_id().is_none() {
                    continue;
                }

                if let Some(name) = record.name() {
                    func(name);
                }
            }
        }
    } else {
        let file = open_file(filename)?;
        let mut parser = parsers::FastqParser::new(vec![file], 10_000, 100_000);
        loop {
            let (block, was_final) = parser.parse()?;
            for read in block.entries {
                func(read.name.get(&block.block));
            }
            if was_final {
                break;
            }
        }
    }
    Ok(())
}

/// Given a fastq or bam file, run a call back on all read sequences
pub fn apply_to_read_sequences(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    let filename = filename.as_ref();
    let ext = filename
        .extension()
        .context("Could not detect filetype from extension")?
        .to_string_lossy();
    if ext == "sam" || ext == "bam" {
        {
            let ignore_unmapped =
                ignore_unmapped.expect("When using bam/sam ignore_unmapped must be set.");
            let mut reader = bam::io::reader::Builder.build_from_path(filename)?;
            reader.read_header()?;
            for result in reader.records() {
                let record = result?;
                if ignore_unmapped && record.reference_sequence_id().is_none() {
                    continue;
                }
                let seq: Vec<u8> = record.sequence().iter().collect();
                func(&seq);
            }
        }
    } else {
        let file = open_file(filename)?;
        let mut parser = parsers::FastqParser::new(vec![file], 10_000, 100_000);
        loop {
            let (block, was_final) = parser.parse()?;
            for read in block.entries {
                func(read.seq.get(&block.block));
            }
            if was_final {
                break;
            }
        }
    }
    Ok(())
}
