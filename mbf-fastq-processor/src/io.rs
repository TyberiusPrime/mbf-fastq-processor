use anyhow::Result;
use ex::fs::File;
use std::{ops::Range, path::Path};

pub mod counting_reader;
pub mod fileformats;
pub mod input;
pub mod output;
pub mod parsers;
pub mod reads;

use crate::config::options::{default_block_size, default_buffer_size};
use crate::config::InputOptions;
pub use input::{
    open_file, open_input_file, open_input_files, DetectedInputFormat, InputFile, InputFiles,
};
pub use reads::{
    longest_suffix_that_is_a_prefix, FastQBlock, FastQBlocksCombined, FastQElement, FastQRead,
    Position, SegmentsCombined, WrappedFastQRead, WrappedFastQReadMut,
};

pub use output::compressed_output;
pub use output::{write_read_to_bam, BamOutput};
pub use parsers::bam_reads_from_index;

pub fn total_file_size(readers: &Vec<File>) -> Option<usize> {
    let mut total = 0;
    for file in readers {
        match file.metadata() {
            Ok(metadata) => {
                total += metadata.len() as usize;
            }
            Err(_) => {
                return None;
            }
        }
    }
    Some(total)
}

/// Given a fastq or bam file, run a call back on all reads
fn apply_to_read(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&Vec<u8>, &FastQRead),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    let filename = filename.as_ref();
    let input_file = open_input_file(filename)?;
    let options = InputOptions {
        fasta_fake_quality: Some(33),
        bam_include_mapped: Some(true),
        bam_include_unmapped: ignore_unmapped.map(|x| !x),
        read_comment_character: b' ', // ignored here.
    };
    let mut parser =
        input_file.get_parser(default_block_size(), default_buffer_size(), &options)?;
    loop {
        let res = parser.parse()?;
        for read in res.fastq_block.entries {
            func(&res.fastq_block.block, &read);
        }
        if res.was_final {
            break;
        }
    }

    Ok(())
}

pub fn apply_to_read_names(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.name.get(block)),
        ignore_unmapped,
    )
}

/// Given a fastq or bam file, run a call back on all read sequences
pub fn apply_to_read_sequences(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.seq.get(block)),
        ignore_unmapped,
    )
}
