use anyhow::{Context, Result};
use std::path::Path;

use crate::get_number_of_cores;
use crate::io::input::InputOptions;
use crate::io::parsers::ThreadCount;
use fastqrab_config::{default_block_size, default_buffer_size};
pub use input::{
    DetectedInputFormat, InputFile, InputFiles, open_file, open_input_file, total_file_size,
};
pub use reads::{
    FastQBlock, FastQBlocksCombined, FastQElement, FastQRead, Position, SegmentsCombined, Tags,
    WrappedFastQRead, WrappedFastQReadMut, longest_suffix_that_is_a_prefix,
};

pub use output::simulated_failure;
pub use parsers::bam_read_count_from_index;

pub mod input;
pub mod output;
pub mod parsers;
pub mod reads;

/// Given a fastq or bam file, run a call back on all reads
fn apply_to_read(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&Vec<u8>, &FastQRead) -> Result<()>,
    include_mapped: bool,
    include_unmapped: bool,
    use_rapidgzip: bool,
) -> Result<()> {
    let filename = filename.as_ref();
    let input_file = open_input_file(filename).context("open_input_file")?;
    let options = InputOptions {
        fasta_fake_quality: Some(33),
        bam_include_mapped: Some(include_mapped),
        bam_include_unmapped: Some(include_unmapped),
        read_comment_character: b' ', // ignored here.
        use_rapidgzip,
        build_rapidgzip_index: None,
        threads_per_segment: Some(get_number_of_cores()), // at this point, we're ready to multicore this
                                                          // hard.
    };
    let mut parser = input_file
        .get_parser(
            default_block_size(),
            default_buffer_size(),
            ThreadCount(std::num::NonZero::new(1usize).expect("1 is not zero")),
            &options,
        )
        .context("Getting parser")?; // cov:excl-line
    loop {
        let res = parser.parse()?;
        for read in res.fastq_block.entries {
            func(&res.fastq_block.block, &read)?;
        }
        if res.was_final {
            break;
        } // cov:excl-line
    }

    Ok(())
}

pub fn apply_to_read_names(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]) -> Result<()>,
    include_mapped: bool,
    include_unmapped: bool,
    use_rapidgzip: bool,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.name.get(block)),
        include_mapped,
        include_unmapped,
        use_rapidgzip,
    )
}

/// Given a fastq or bam file, run a call back on all read sequences
pub fn apply_to_read_sequences(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8]) -> Result<()>,
    include_mapped: bool,
    include_unmapped: bool,
    use_rapidgzip: bool,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.seq.get(block)),
        include_mapped,
        include_unmapped,
        use_rapidgzip,
    )
}

/// Given a FASTA or FASTQ file or BAM file, run a callback on each read's (name, sequence) pair.
/// For FASTA, the name is the record id (and description if present), without the leading `>`.
pub fn apply_to_read_names_and_sequences(
    filename: impl AsRef<Path>,
    func: &mut impl FnMut(&[u8], &[u8]) -> Result<()>,
    use_rapidgzip: bool,
) -> Result<()> {
    apply_to_read(
        filename,
        &mut |block: &Vec<u8>, read: &FastQRead| func(read.name.get(block), read.seq.get(block)),
        true,
        true,
        use_rapidgzip,
    )
}
