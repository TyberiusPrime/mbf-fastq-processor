use anyhow::Result;
use std::{ops::Range, path::Path};

pub mod fileformats;
pub mod input;
pub mod output;
pub mod parsers;
pub mod reads;

use crate::config::InputOptions;
use crate::config::options::{default_block_size, default_buffer_size};
pub use input::{
    DetectedInputFormat, InputFile, InputFiles, open_file, open_input_file, open_input_files,
};
pub use reads::{
    FastQBlock, FastQBlocksCombined, FastQElement, FastQRead, Position, SegmentsCombined,
    WrappedFastQRead, WrappedFastQReadMut, longest_suffix_that_is_a_prefix,
};

pub use output::compressed_output;
pub use output::{BamOutput, write_read_to_bam};

/// Given a fastq or bam file, run a call back on all reads
fn apply_to_read(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&Vec<u8>, &FastQRead),
    ignore_unmapped: Option<bool>,
) -> Result<()> {
    let filename = filename.as_ref();
    let input_file = open_input_file(filename, false, 1)?;
    let options = InputOptions {
        fasta_fake_quality: Some(33),
        bam_include_mapped: Some(true),
        bam_include_unmapped: ignore_unmapped.map(|x| !x),
        read_comment_character: b' ', // ignored here.
        use_internal_rapidgzip: false,
    };
    let mut parser =
        input_file.get_parser(default_block_size(), default_buffer_size(), &options)?;
    loop {
        let (block, was_final) = parser.parse()?;
        for read in block.entries {
            func(&block.block, &read);
        }
        if was_final {
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
