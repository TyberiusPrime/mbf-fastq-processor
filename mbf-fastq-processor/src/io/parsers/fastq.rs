use super::{ParseResult, Parser};
use crate::io::{FastQBlock, FastQElement, FastQRead, Position};
use anyhow::{Context, Result, bail};
use bstr::BString;
use niffler;
use std::io::Read;

pub struct FastqParser {
    current_reader: Box<dyn Read + Send>,
    current_block: Option<FastQBlock>,
    buf_size: usize,
    target_reads_per_block: usize,
    last_partial: Option<FastQRead>,
    last_status: PartialStatus,
    windows_mode: Option<bool>,
    compression_format: niffler::send::compression::Format,
}

impl FastqParser {
    #[must_use]
    pub fn new(file: std::fs::File, target_reads_per_block: usize, buf_size: usize) -> Result<FastqParser> {
        let (reader, format) = niffler::send::get_reader(Box::new(file))?;

        Ok(FastqParser {
            current_reader: reader,
            current_block: Some(FastQBlock {
                block: Vec::new(),
                entries: Vec::new(),
            }),
            buf_size,
            target_reads_per_block,
            last_partial: None,
            last_status: PartialStatus::NoPartial,
            windows_mode: None,
            compression_format: format,
        })
    }

    fn advance(&mut self, start: &mut usize) -> Result<bool> {
        {
            if *start >= self.current_block.as_ref().unwrap().block.len() {
                self.current_block
                    .as_mut()
                    .unwrap()
                    .block
                    .extend(vec![0; self.buf_size]);
            }

            let read = self
                .current_reader
                .read(&mut self.current_block.as_mut().unwrap().block[*start..])?;

            if read == 0 {
                return Ok(false);
            }
            *start += read;
        }
        return Ok(true);
    }

    fn next_block(&mut self) -> Result<(FastQBlock, bool)> {
        let mut was_final = false;
        let mut start = self.current_block.as_ref().unwrap().block.len();
        while self.current_block.as_ref().unwrap().entries.len() < self.target_reads_per_block {
            let block_start = start;

            if self.windows_mode.is_none() {
                if !self.advance(&mut start)? {
                    //empty file
                    was_final = true;
                    break;
                }
                while self.windows_mode.is_none() {
                    let block = &self.current_block.as_ref().unwrap().block;
                    if memchr::memmem::find(block, b"\r\n").is_some() {
                        self.windows_mode = Some(true);
                        break;
                    } else if memchr::memchr(b'\n', block).is_some() {
                        self.windows_mode = Some(false);
                        break;
                    }
                    //when the bufsize is smaller than the first read name, we need to read more.
                    //pathological? yes.
                    if !self.advance(&mut start)? {
                        panic!("Read all of file, but found no newlines");
                    }
                }

                //read until we have at least one newline.
            } else {
                if !self.advance(&mut start)? {
                    was_final = true;
                    break;
                }
            }
            let parse_result = parse_to_fastq_block(
                self.current_block.as_mut().unwrap(),
                block_start,
                start,
                self.last_status,
                self.last_partial.take(),
                self.windows_mode
                    .expect("Window mode must be set at this point"),
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
    fn parse(&mut self) -> Result<ParseResult> {
        let (block, was_final) = self.next_block()?;
        Ok(ParseResult {
            fastq_block: block,
            was_final,
        })
    }

    fn bytes_per_base(&self) -> f64 {
        match self.compression_format {
            niffler::send::compression::Format::Gzip => 0.5,
            niffler::send::compression::Format::Bzip => 0.5,
            niffler::send::compression::Format::Lzma => 0.5,
            niffler::send::compression::Format::Zstd => 0.5,
            niffler::send::compression::Format::No => 2.25,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PartialStatus {
    NoPartial,
    InName,
    InSeq,
    InSpacer,
    InQual,
    InNameNewline,   //only in windows mode
    InSeqNewline,    //only in windows mode
    InSpacerNewline, //only in windows mode
    InQualNewline,   //only in windows mode
}

pub struct FastQBlockParseResult {
    //pub block: FastQBlock,
    pub status: PartialStatus,
    pub partial_read: Option<FastQRead>,
    pub windows_mode: bool,
}

#[allow(clippy::too_many_lines)]
pub fn parse_to_fastq_block(
    target_block: &mut FastQBlock,
    mut start_offset: usize,
    stop: usize,
    last_status: PartialStatus,
    last_read: Option<FastQRead>,
    windows_mode: bool,
) -> Result<FastQBlockParseResult> {
    let org_status = last_status;
    let input = &mut target_block.block;
    let entries = &mut target_block.entries;
    let mut pos = start_offset;
    //debug!("start offset is {pos}");
    let mut last_status = last_status;
    let mut last_read = last_read;
    let (mut newline_iterator, newline_length) = if windows_mode {
        //debug!("new extended block {last_status:?}");
        let verify_newline = match last_status {
            PartialStatus::InNameNewline => {
                last_status = PartialStatus::InSeq;
                true
            }
            PartialStatus::InSeqNewline => {
                last_status = PartialStatus::InSpacer;
                true
            }
            PartialStatus::InSpacerNewline => {
                last_status = PartialStatus::InQual;
                true
            }
            PartialStatus::InQualNewline => {
                last_status = PartialStatus::NoPartial;
                true
            }
            _ => false,
        };
        if verify_newline {
            // debug!("Started within newline");
            if input[pos] != b'\n' {
                bail!("Expected \\n after \\r in windows mode. Failed around position {pos}");
            }
            pos += 1;
            start_offset += 1;
        }
        (memchr::memmem::find_iter(&input[pos..stop], b"\r\n"), 2)
    } else {
        (memchr::memmem::find_iter(&input[pos..stop], b"\n"), 1)
    };
    let start_offset = start_offset;

    if last_status == PartialStatus::InName {
        let last_read2 = last_read.as_mut().unwrap();
        let next_newline = newline_iterator.next();
        // debug!("Continue reading inname Next_newline: {next_newline:?}");
        match next_newline {
            Some(next_newline) => {
                match &mut last_read2.name {
                    FastQElement::Owned(name) => {
                        name.extend_from_slice(&input[pos..start_offset + next_newline]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                pos = start_offset + next_newline + newline_length;
                last_status = PartialStatus::InSeq;
            }
            None => {
                let (status, name_end) = if windows_mode && input[stop - 1] == b'\r' {
                    (PartialStatus::InNameNewline, stop - 1)
                } else {
                    (PartialStatus::InName, stop)
                };

                match &mut last_read2.name {
                    FastQElement::Owned(name) => {
                        name.extend_from_slice(&input[pos..name_end]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                // debug!("Returning in name 1 {:?}", last_read.as_ref().unwrap());
                return Ok(FastQBlockParseResult {
                    status: status,
                    partial_read: Some(last_read.unwrap()),
                    windows_mode,
                });
            }
        }
        // debug!( "Continue reading name: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[..next_newline]).unwrap());
    }
    if PartialStatus::InSeq == last_status {
        let last_read2 = last_read.as_mut().unwrap();
        let next_newline = newline_iterator.next();
        // debug!("Continue reading inseq Next_newline: {next_newline:?}");
        match next_newline {
            Some(next_newline) => {
                match &mut last_read2.seq {
                    FastQElement::Owned(seq) => {
                        seq.extend_from_slice(&input[pos..start_offset + next_newline]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                pos = start_offset + next_newline + newline_length;
            }
            None => {
                let (status, seq_end) = if windows_mode && input[stop - 1] == b'\r' {
                    (PartialStatus::InSeqNewline, stop - 1)
                } else {
                    (PartialStatus::InSeq, stop)
                };

                match &mut last_read2.seq {
                    FastQElement::Owned(seq) => {
                        seq.extend_from_slice(&input[pos..seq_end]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                // debug!("Returning in seq1: {:?}", last_read.as_ref().unwrap());
                return Ok(FastQBlockParseResult {
                    status: status,
                    partial_read: Some(last_read.unwrap()),
                    windows_mode,
                });
            }
        }
        if pos < stop && input[pos] != b'+' {
            bail!(
                "Expected + after sequence in input. Position {pos}, was {}, Read name was: '{}'.\nIf your Fastq is line-wrapped, sorry that's not supported.",
                input[pos],
                BString::from(last_read2.name.get(input))
            );
        }
        last_status = PartialStatus::InSpacer;
    }
    if PartialStatus::InSpacer == last_status {
        let next_newline = newline_iterator.next();
        match next_newline {
            Some(next_newline) => {
                /* debug!(
                    "Continue reading spacer: {next_newline} {} {}",
                    input.len(),
                    std::str::from_utf8(&input[pos..pos + next_newline]).unwrap()
                ); */
                pos = start_offset + next_newline + newline_length;
            }
            None => {
                let status = if windows_mode && input[stop - 1] == b'\r' {
                    PartialStatus::InSpacerNewline
                } else {
                    PartialStatus::InSpacer
                };

                // debug!("Returning in spacer");
                return Ok(FastQBlockParseResult {
                    status: status,
                    partial_read: Some(last_read.unwrap()),
                    windows_mode,
                });
            }
        }

        last_status = PartialStatus::InQual;
    }
    if PartialStatus::InQual == last_status {
        let last_read2 = last_read.as_mut().unwrap();
        let next_newline = newline_iterator.next();
        match next_newline {
            Some(next_newline) => {
                // println!(
                //     "Continue reading qual: {next_newline} {} {}. First byte: {}. newline byte: {}. windows mode: {}",
                //     input.len(),
                //     std::str::from_utf8(&input[pos..start_offset + next_newline]).unwrap(),
                //     input[start_offset],
                //     input[start_offset + next_newline],
                //     windows_mode
                // );
                match &mut last_read2.qual {
                    FastQElement::Owned(qual) => {
                        qual.extend_from_slice(&input[pos..start_offset + next_newline]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                pos = start_offset + next_newline + newline_length;
            }
            None => {
                let (status, qual_end) = if windows_mode && input[stop - 1] == b'\r' {
                    (PartialStatus::InQualNewline, stop - 1)
                } else {
                    (PartialStatus::InQual, stop)
                };

                match &mut last_read2.qual {
                    FastQElement::Owned(qual) => {
                        qual.extend_from_slice(&input[pos..qual_end]);
                    }
                    FastQElement::Local(_) => panic!("Should not happen"),
                }
                return Ok(FastQBlockParseResult {
                    status: status,
                    partial_read: Some(last_read.unwrap()),
                    windows_mode,
                });
            }
        }
    }
    if let Some(last_read) = last_read {
        last_read.verify().with_context(|| {
            format!(
                "Read was: \nname: {}\n seq: '{}' (len={})\nqual: '{}' (len={}).\nPosition around {}. Org status: {:?}",
                BString::from(last_read.name.get(input)),
                BString::from(last_read.seq.get(input)),
                last_read.seq.get(input).len(),
                BString::from(last_read.qual.get(input)),
                last_read.qual.get(input).len(),
                pos,
                org_status

            )
        })?;

        entries.push(last_read);
    }

    //read full reads until last (possibly partial red)

    let mut status = PartialStatus::NoPartial;
    let mut partial_read: Option<FastQRead> = None;
    // debug!("before loop pos {pos} stop {stop}");

    loop {
        if pos >= stop {
            break;
        }
        if input[pos] != b'@' {
            if pos == stop - 1 && input[pos] == b'\n' {
                // empty new line at end of file, ignore. test case is in
                // test_trim_adapter_mismatch_tail
                break;
            } else {
                let letter: BString = (&input[pos..=pos]).into();
                bail!(
                    "Unexpected symbol where @ was expected in input. Position {}, was '{}' (0x{:x}). Check your fastq",
                    pos,
                    letter,
                    input[pos]
                );
            }
        }
        let end_of_name = newline_iterator.next();
        let (name_start, name_end) = match end_of_name {
            Some(end_of_name) => {
                let r = (pos + 1, end_of_name + start_offset);
                assert!((r.0 < r.1), "Empty name");
                pos = start_offset + end_of_name + newline_length;
                r
            }
            None => {
                let name_end = if windows_mode && input[stop - 1] == b'\r' {
                    status = PartialStatus::InNameNewline;
                    stop - 1
                } else {
                    status = PartialStatus::InName;
                    stop
                };
                partial_read = Some(
                    FastQRead::new(
                        FastQElement::Owned(input[pos + 1..name_end].to_vec()),
                        FastQElement::Owned(Vec::new()),
                        FastQElement::Owned(Vec::new()),
                    )
                    .unwrap(),
                );
                break;
            }
        };
        let end_of_seq = newline_iterator.next();
        let (seq_start, seq_end) = match end_of_seq {
            Some(end_of_seq) => {
                let r = (pos, end_of_seq + start_offset);
                pos = start_offset + end_of_seq + newline_length;
                r
            }
            None => {
                let seq_end = if windows_mode && input[stop - 1] == b'\r' {
                    status = PartialStatus::InSeqNewline;
                    stop - 1
                } else {
                    status = PartialStatus::InSeq;
                    stop
                };
                partial_read = Some(FastQRead {
                    // can't call new, we must not verify here, verify later
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[pos..seq_end].to_vec()),
                    qual: FastQElement::Owned(Vec::new()),
                });
                // debug!("Returning in seq2 {:?}", partial_read.as_ref().unwrap());
                break;
            }
        };
        if pos < stop && input[pos] != b'+' {
            bail!(
                "Expected + after sequence in FastQ input, got {} at position {}",
                input[pos],
                pos
            );
        }
        let end_of_spacer = newline_iterator.next();
        match end_of_spacer {
            Some(end_of_spacer) => {
                pos = start_offset + end_of_spacer + newline_length;
            }
            None => {
                if windows_mode && input[stop - 1] == b'\r' {
                    status = PartialStatus::InSpacerNewline;
                } else {
                    status = PartialStatus::InSpacer;
                }
                partial_read = Some(FastQRead {
                    // can't call new, must not verify yet
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[seq_start..seq_end].to_vec()),
                    qual: FastQElement::Owned(Vec::new()),
                });
                // debug!("Returning in spacer {:?}", partial_read.as_ref().unwrap());
                break;
            }
        }
        let end_of_qual = newline_iterator.next();
        let (qual_start, qual_end) = match end_of_qual {
            Some(end_of_qual) => {
                let r = (pos, end_of_qual + start_offset);
                pos = start_offset + end_of_qual + newline_length;
                r
            }
            None => {
                let qual_end = if windows_mode && input[stop - 1] == b'\r' {
                    status = PartialStatus::InQualNewline;
                    stop - 1
                } else {
                    status = PartialStatus::InQual;
                    stop
                };
                partial_read = Some(FastQRead {
                    // can't call new, must not verify yet
                    name: FastQElement::Owned(input[name_start..name_end].to_vec()),
                    seq: FastQElement::Owned(input[seq_start..seq_end].to_vec()),
                    qual: FastQElement::Owned(input[pos..qual_end].to_vec()),
                });

                // debug!("Returning in qual {:?}", partial_read.as_ref().unwrap());
                break;
            }
        };
        entries.push(
            FastQRead::new(
                FastQElement::Local(Position {
                    start: name_start,
                    end: name_end,
                }),
                FastQElement::Local(Position {
                    start: seq_start,
                    end: seq_end,
                }),
                FastQElement::Local(Position {
                    start: qual_start,
                    end: qual_end,
                }),
            )
            .with_context(|| {
                format!(
                    " in read '{name}', near position: {pos}",
                    name = BString::from(&input[name_start..name_end])
                )
            })?,
        );
    }

    Ok(FastQBlockParseResult {
        status,
        partial_read,
        windows_mode,
    })
}
