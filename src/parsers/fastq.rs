use crate::io::{FastQBlock, FastQRead, PartialStatus, parse_to_fastq_block};
use crate::parsers::Parser;
use anyhow::{Context, Result, bail};
use ex::fs::File;
use niffler;
use std::io::Read;

pub struct FastqParser {
    readers: Vec<File>,
    current_reader: Option<Box<dyn Read + Send>>,
    current_block: Option<FastQBlock>,
    buf_size: usize,
    target_reads_per_block: usize,
    last_partial: Option<FastQRead>,
    last_status: PartialStatus,
    windows_mode: Option<bool>,
}

impl FastqParser {
    #[must_use]
    pub fn new(
        mut readers: Vec<File>,
        target_reads_per_block: usize,
        buf_size: usize,
    ) -> FastqParser {
        readers.reverse(); // so we can pop() them one by one in the right order
        FastqParser {
            readers,
            current_reader: None,
            current_block: Some(FastQBlock {
                block: Vec::new(),
                entries: Vec::new(),
            }),
            buf_size,
            target_reads_per_block,
            last_partial: None,
            last_status: PartialStatus::NoPartial,
            windows_mode: None,
        }
    }

    fn next_block(&mut self) -> Result<(FastQBlock, bool)> {
        let mut was_final = false;
        let mut start = self.current_block.as_ref().unwrap().block.len();
        while self.current_block.as_ref().unwrap().entries.len() < self.target_reads_per_block {
            if self.current_reader.is_none() {
                if let Some(next_file) = self.readers.pop() {
                    let (reader, _format) = niffler::send::get_reader(Box::new(next_file))?;
                    self.current_reader = Some(reader);
                } else {
                    unreachable!();
                };
            }

            let block_start = start;
            if start >= self.current_block.as_ref().unwrap().block.len() {
                self.current_block
                    .as_mut()
                    .unwrap()
                    .block
                    .extend(vec![0; self.buf_size]);
            }

            let read = self
                .current_reader
                .as_mut()
                .expect("current_reader must exist when reading")
                .read(&mut self.current_block.as_mut().unwrap().block[start..])?;

            if read == 0 {
                self.windows_mode = None;
                self.current_reader = None;
                if self.readers.is_empty() {
                    was_final = true;
                    break;
                }
                continue;
            }
            start += read;
            let parse_result = parse_to_fastq_block(
                self.current_block.as_mut().unwrap(),
                block_start,
                start,
                self.last_status,
                self.last_partial.take(),
                self.windows_mode,
            )?;
            self.last_status = parse_result.status;
            self.last_partial = parse_result.partial_read;
            self.windows_mode = Some(parse_result.windows_mode);
        }
        self.current_block.as_mut().unwrap().block.resize(start, 0);

        let (mut out_block, new_block) = self
            .current_block
            .take()
            .unwrap()
            .split_at(self.target_reads_per_block);

        self.current_block = Some(new_block);
        if was_final {
            if let Some(partial) = self.last_partial.take() {
                match self.last_status {
                    PartialStatus::InQual => {}
                    PartialStatus::NoPartial => unreachable!(),
                    _ => bail!("Incomplete final read. Was in state {:?}", self.last_status),
                }
                let final_read = FastQRead::new(partial.name, partial.seq, partial.qual)
                    .context("In parsing final read")?;
                out_block.entries.push(final_read);
            }
        }
        Ok((out_block, was_final))
    }
}

impl Parser for FastqParser {
    fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        self.next_block()
    }
}
