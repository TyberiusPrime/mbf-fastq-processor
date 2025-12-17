use crate::config::InputOptions;
use crate::io::{FastQBlock, InputFile};
use anyhow::Result;

mod bam;
mod fasta;
mod fastq;

pub use bam::{BamParser, bam_reads_from_index};
pub use fasta::FastaParser;
pub use fastq::FastqParser;

pub struct ParseResult {
    pub fastq_block: FastQBlock,
    pub was_final: bool,
}

pub trait Parser: Send {
    fn parse(&mut self) -> Result<ParseResult>;
    fn bytes_per_base(&self) -> f64;
}

#[derive(Clone, Copy, Debug)]
pub struct ThreadCount(pub usize); //todo: replace with non-zero

///parse multiple files one after the other
///this allows the mixing of input file types, I suppose.
pub struct ChainedParser {
    pending: Vec<InputFile>,
    current: Option<Box<dyn Parser>>,
    bam_index_paths: Option<Vec<std::path::PathBuf>>,
    target_reads_per_block: usize,
    buffer_size: usize,
    input_thread_count: ThreadCount,
    options: InputOptions,
    expected_read_count: Option<usize>,
    first_block_done: bool,
    total_input_file_size: Option<u64>,
}

pub struct ChainParseResult {
    pub fastq_block: FastQBlock,
    pub was_final: bool,
    pub expected_read_count: Option<usize>,
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn calc_next_power_of_two(total: usize) -> usize {
    2usize.pow((total as f64).log2().ceil() as u32)
}

impl ChainedParser {
    #[must_use]
    pub fn new(
        mut files: Vec<InputFile>,
        target_reads_per_block: usize,
        buffer_size: usize,
        input_thread_count: ThreadCount,
        options: InputOptions,
    ) -> Self {
        files.reverse();
        let bam_index_paths = files
            .iter()
            .filter_map(|file| {
                if let InputFile::Bam(_, index_path) = file {
                    Some(index_path.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let total_input_file_size = super::input::total_file_size(&files);

        ChainedParser {
            pending: files,
            current: None,
            bam_index_paths: if bam_index_paths.is_empty() {
                None
            } else {
                Some(bam_index_paths)
            },
            target_reads_per_block,
            buffer_size,
            input_thread_count,
            options,
            expected_read_count: None,
            first_block_done: false,
            total_input_file_size,
        }
    }

    fn ensure_parser(&mut self) -> Result<bool> {
        while self.current.is_none() {
            match self.pending.pop() {
                Some(file) => {
                    let parser = file.get_parser(
                        self.target_reads_per_block,
                        self.buffer_size,
                        self.input_thread_count,
                        &self.options,
                    )?;
                    self.current = Some(parser);
                }
                None => return Ok(false),
            }
        }
        Ok(true)
    }

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_sign_loss)]
    pub fn parse(&mut self) -> Result<ChainParseResult> {
        loop {
            if !self.ensure_parser()? {
                return Ok(ChainParseResult {
                    fastq_block: FastQBlock {
                        block: Vec::new(),
                        entries: Vec::new(),
                    },
                    was_final: true,
                    expected_read_count: self.expected_read_count,
                });
            }

            let mut res = self
                .current
                .as_mut()
                .expect("parser must exist after ensure_parser")
                .parse()?;

            if !self.first_block_done {
                //this is where we need to implement the exact expected read count.
                //We have the first block, with an average read length,
                //and from there and a basic assumption on compression,
                //we can work out how many reads we expect in the rest of the files.
                self.first_block_done = true;
                match &self.bam_index_paths {
                    Some(paths) => {
                        let total: Option<usize> = paths
                            .iter()
                            .map(|path| {
                                bam_reads_from_index(
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
                        let next_power_of_two = total.map(calc_next_power_of_two);
                        self.expected_read_count = next_power_of_two;
                    }
                    None => {
                        let reads_so_far = res.fastq_block.entries.len();
                        if reads_so_far > 0 {
                            //sheer paranoia, but downstream has to cope with this being
                            //unknown anyway for non-file inputs
                            if let Some(total_input_file_size) = self.total_input_file_size {
                                let avg_read_length =
                                    res.fastq_block
                                        .entries
                                        .iter()
                                        .map(|e| e.seq.len())
                                        .sum::<usize>() as f64
                                        / reads_so_far as f64;
                                let bytes_per_base = self
                                    .current
                                    .as_ref()
                                    .expect("Current always set at this place")
                                    .bytes_per_base();
                                let expected_reads = total_input_file_size as f64
                                    / (avg_read_length * bytes_per_base);
                                let next_power_of_two =
                                    calc_next_power_of_two(expected_reads as usize);
                                self.expected_read_count = Some(next_power_of_two);
                                /* dbg!(
                                    avg_read_length,
                                    bytes_per_base,
                                    total_input_file_size,
                                    expected_reads
                                ); */
                            }
                        }
                    }
                }
            }

            if res.was_final {
                self.current = None; //so the next entry will load a new parser.
                if !self.pending.is_empty() {
                    res.was_final = false;
                }
            }

            if res.fastq_block.entries.is_empty() && !res.was_final {
                //when does this happen
                //I think perhaps, when we had not enough bytes to fill a single read left over in this
                //block?
                continue;
            }
            return Ok(ChainParseResult {
                fastq_block: res.fastq_block,
                was_final: res.was_final,
                expected_read_count: self.expected_read_count,
            });
        }
    }
}
