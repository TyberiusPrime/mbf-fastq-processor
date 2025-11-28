use anyhow::Result;
use ex::fs::File;
use std::{ops::Range, path::Path};

pub mod fileformats;
pub mod input;
pub mod output;
pub mod parsers;
pub mod reads;

use crate::config::InputOptions;
use crate::config::options::{default_block_size, default_buffer_size};
use crate::io::parsers::ThreadCount;
pub use input::{
    DetectedInputFormat, InputFile, InputFiles, open_file, open_input_file, open_input_files,
};
pub use reads::{
    FastQBlock, FastQBlocksCombined, FastQElement, FastQRead, Position, SegmentsCombined,
    WrappedFastQRead, WrappedFastQReadMut, longest_suffix_that_is_a_prefix,
};

pub use output::compressed_output;
pub use output::{BamOutput, write_read_to_bam};
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
    include_mapped: bool,
    include_unmapped: bool,
) -> Result<()> {
    let filename = filename.as_ref();
    let input_file = open_input_file(filename)?;
    let options = InputOptions {
        fasta_fake_quality: Some(33),
        bam_include_mapped: Some(include_mapped),
        bam_include_unmapped: Some(include_unmapped),
        read_comment_character: b' ', // ignored here.
        use_rapidgzip: Some(false),   //todo : should we use the config here?
        build_rapidgzip_index: None,
    };
    let mut parser =
        input_file.get_parser(default_block_size(), default_buffer_size(), ThreadCount(1), &options)?;
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
    include_mapped: bool,
    include_unmapped: bool,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.name.get(block)),
        include_mapped,
        include_unmapped,
    )
}

/// Given a fastq or bam file, run a call back on all read sequences
pub fn apply_to_read_sequences(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]),
    include_mapped: bool,
    include_unmapped: bool,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.seq.get(block)),
        include_mapped,
        include_unmapped,
    )
}
