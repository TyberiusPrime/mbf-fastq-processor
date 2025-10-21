use crate::config::InputOptions;
use crate::io::{FastQBlock, InputFile};
use anyhow::Result;

mod bam;
mod fasta;
mod fastq;

pub use bam::BamParser;
pub use fasta::FastaParser;
pub use fastq::FastqParser;

pub trait Parser: Send {
    fn parse(&mut self) -> Result<(FastQBlock, bool)>;
}

///parse multiple files one after the other
pub struct ChainedParser {
    pending: Vec<InputFile>,
    current: Option<Box<dyn Parser>>,
    target_reads_per_block: usize,
    buffer_size: usize,
    options: InputOptions,
}

impl ChainedParser {
    #[must_use]
    pub fn new(
        mut files: Vec<InputFile>,
        target_reads_per_block: usize,
        buffer_size: usize,
        options: InputOptions,
    ) -> Self {
        files.reverse();
        ChainedParser {
            pending: files,
            current: None,
            target_reads_per_block,
            buffer_size,
            options,
        }
    }

    fn ensure_parser(&mut self) -> Result<bool> {
        while self.current.is_none() {
            match self.pending.pop() {
                Some(file) => {
                    let parser = file.get_parser(
                        self.target_reads_per_block,
                        self.buffer_size,
                        &self.options,
                    )?;
                    self.current = Some(parser);
                }
                None => return Ok(false),
            }
        }
        Ok(true)
    }
}

impl Parser for ChainedParser {
    fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        loop {
            if !self.ensure_parser()? {
                return Ok((
                    FastQBlock {
                        block: Vec::new(),
                        entries: Vec::new(),
                    },
                    true,
                ));
            }

            let (block, mut finished) = self
                .current
                .as_mut()
                .expect("parser must exist after ensure_parser")
                .parse()?;

            if finished {
                self.current = None;
                if !self.pending.is_empty() {
                    finished = false;
                }
            }

            if block.entries.is_empty() && !finished {
                continue;
            }

            return Ok((block, finished));
        }
    }
}
