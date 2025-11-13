//! File I/O, parsers, and output formatting for mbf-fastq-processor
//!
//! This crate handles:
//! - Reading FastQ files in multiple formats (raw, gzip, zstd)
//! - Parsing FastQ and BAM files
//! - Writing compressed output
//! - File format detection
//! - Parallel I/O operations

#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

pub mod fileformats;
pub mod input;
pub mod output;
pub mod parsers;

pub use mbf_core::{FastQRead, FastQElement, Position, WrappedFastQRead, WrappedFastQReadMut};

pub use input::{
    DetectedInputFormat, InputFile, InputFiles, open_file, open_input_file, open_input_files,
};

pub use output::compressed_output;
pub use output::{BamOutput, write_read_to_bam};
