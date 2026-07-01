use anyhow::Result;
use std::num::NonZero;
use std::path::PathBuf;

use crate::blocks::FastQChunk;
use crate::io::InputFile;
use crate::io::input::InputOptions;

mod bam;
mod fasta;
mod fastq;
#[cfg(unix)]
mod shm;

pub use bam::{BamParser, bam_read_count_from_index};
pub use fasta::FastaParser;
pub use fastq::PodFastqParser;

pub struct ParseResult {
    pub output: FastQChunk,
    pub was_final: bool,
}

pub trait Parser: Send {
    fn parse(&mut self) -> Result<ParseResult>;
    fn bytes_per_base(&self) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct ThreadCount(pub std::num::NonZero<usize>);

/// The per-segment thread budget for one input parser. `decompression` sizes the
/// rapidgzip `-P` decode pool (and the BAM bgzf workers); `pod_demux` sizes the
/// columnar pod-parser's demux pool. They are tuned independently — decode wants
/// many threads, demux wants a small handful — but both originate from
/// `calculate_thread_counts`, the single place these are decided.
#[derive(Clone, Copy, Debug)]
pub struct ParserThreadCounts {
    pub decompression: ThreadCount,
    pub pod_demux: ThreadCount,
}

///parse multiple files one after the other
///this allows the mixing of input file types, I suppose.
pub struct ChainedParser {
    pending: Vec<InputFile>,
    current: Option<Box<dyn Parser>>,
    current_filename: Option<PathBuf>,
    bam_index_paths: Option<Vec<std::path::PathBuf>>,
    target_reads_per_block: NonZero<usize>,
    buffer_size: usize,
    thread_counts: ParserThreadCounts,
    options: InputOptions,
    expected_read_count_power_of_two: Option<usize>,
    first_block_done: bool,
    total_input_file_size: Option<u64>,
    reads_so_far: usize,
}

pub struct ChainParseResult {
    pub fastq_block: FastQChunk,
    pub was_final: bool,
    pub expected_read_count: Option<usize>,
}

impl ChainedParser {
    #[must_use]
    pub fn new(
        mut files: Vec<InputFile>,
        target_reads_per_block: NonZero<usize>,
        buffer_size: usize,
        thread_counts: ParserThreadCounts,
        options: InputOptions,
    ) -> Self {
        files.reverse();
        let bam_index_paths = files
            .iter()
            .filter_map(|file| {
                if let InputFile::Bam(_, index_path) = file {
                    // `None` for handle-only BAM inputs that carry no path; those
                    // simply contribute no index path.
                    index_path.clone()
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let total_input_file_size = super::input::total_file_size(&files);

        ChainedParser {
            pending: files,
            current: None,
            current_filename: None,
            bam_index_paths: if bam_index_paths.is_empty() {
                None
            } else {
                Some(bam_index_paths)
            },
            target_reads_per_block,
            buffer_size,
            thread_counts,
            options,
            expected_read_count_power_of_two: None,
            first_block_done: false,
            total_input_file_size,
            reads_so_far: 0,
        }
    }

    fn ensure_parser(&mut self) -> Result<bool> {
        while self.current.is_none() {
            match self.pending.pop() {
                Some(file) => {
                    self.current_filename = file.get_filename().cloned();
                    let parser = file.get_parser(
                        self.target_reads_per_block,
                        self.buffer_size,
                        self.thread_counts,
                        &self.options,
                    )?; // cov:excl-line - would only happen if the file had something happen to it
                    // between format detection & opening it for output
                    // possible, since we're format detecting everything at the start,
                    // but not worth a test for coverag.
                    self.current = Some(parser);
                }
                None => return Ok(false), // cov:excl-line -- see below
            }
        }
        Ok(true)
    }

    /// # Panics
    /// When `ensure_parser` doesn't ensure
    /// when bam options not set correctly by validation
    #[expect(
        clippy::cast_sign_loss,
        reason = "Expected_reads -> usize, is always positive"
    )]
    pub fn parse(&mut self) -> Result<ChainParseResult> {
        if !self.ensure_parser()? {
            //cov:excl-start
            //see ensure_parser comment
            return Ok(ChainParseResult {
                fastq_block: FastQChunk::new_empty(),
                was_final: true,
                expected_read_count: self.expected_read_count_power_of_two,
            });
            //cov:excl-end
        }

        let res = self
            .current
            .as_mut()
            .expect("parser must exist after ensure_parser")
            .parse()?;
        let mut was_final = res.was_final;
        let output = res.output;

        if !self.first_block_done {
            //this is where we need to implement the exact expected read count.
            //We have the first block, with an average read length,
            //and from there and a basic assumption on compression,
            //we can work out how many reads we expect in the rest of the files.
            self.first_block_done = true;
            if let Some(paths) = &self.bam_index_paths {
                let total: Option<usize> = paths
                    .iter()
                    .map(|path| {
                        bam_read_count_from_index(
                            path,
                            self.options
                                .bam_include_mapped
                                .expect("must have been set by validation"),
                            self.options
                                .bam_include_unmapped
                                .expect("must have been set by validation"),
                        )
                    })
                    .sum();
                let next_power_of_two = total.map(usize::next_power_of_two);
                self.expected_read_count_power_of_two = next_power_of_two;
            } else {
                //this happens for non-bam files!
                let reads_so_far = output.row_count();
                // A 0-read first block is legitimate when the input is empty or
                // the out-of-process decoder failed before emitting a record (e.g.
                // a truncated gzip): skip the estimation and let the real error
                // surface downstream (the decompressor's non-zero exit / reader
                // thread), rather than asserting here.
                if reads_so_far > 0 {
                    //sheer paranoia, but downstream has to cope with this being
                    //unknown anyway for non-file inputs
                    #[expect(
                        clippy::cast_precision_loss,
                        clippy::cast_possible_truncation,
                        reason = "entries is going to be smaller than 2**52"
                    )]
                    if let Some(total_input_file_size) = self.total_input_file_size {
                        let avg_read_length = output.total_seq_len() as f64 / reads_so_far as f64;
                        let bytes_per_base = self
                            .current
                            .as_ref()
                            .expect("Current always set at this place")
                            .bytes_per_base();
                        let expected_reads =
                            total_input_file_size as f64 / (avg_read_length * bytes_per_base);
                        let expected_reads = expected_reads.ceil() as usize;
                        let next_power_of_two = expected_reads.next_power_of_two();
                        self.expected_read_count_power_of_two = Some(next_power_of_two);
                        /* dbg!(
                            avg_read_length,
                            bytes_per_base,
                            total_input_file_size,
                            expected_reads
                        ); */
                    }
                } // cov:excl-line
            }
        }

        if was_final {
            self.current = None; //so the next entry will load a new parser from pending.
            if !self.pending.is_empty() {
                was_final = false;
            }
        }

        let fastq_block = output;
        self.reads_so_far += fastq_block.row_count();
        Ok(ChainParseResult {
            fastq_block,
            was_final,
            expected_read_count: self.expected_read_count_power_of_two,
        })
    }
}
