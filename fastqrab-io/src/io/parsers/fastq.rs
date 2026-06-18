use anyhow::{Context, Result, bail};
use bstr::{BString, ByteSlice};
use niffler;
use std::collections::VecDeque;
use std::num::NonZero;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{io::Read, path::PathBuf};

use crossbeam::channel::{self, Receiver};
use stringpod::{DualStringPodBuilder, StringPod, StringPodBuilder};

use crate::blocks::FastQChunk;
use crate::io::parsers::{ParseResult, Parser, ParserOutput};
use crate::io::pod_parser::{FastqChunk as PodFastqChunk, parse_pods_from_channel};
use crate::io::{
    FastQBlock, FastQElement, FastQRead, Position,
    input::{DecompressionOptions, open_decompressed_reader},
};

pub struct FastqParser {
    current_reader: Box<dyn Read + Send>,
    current_block: Option<FastQBlock>,
    buf_size: usize,
    target_reads_per_block: NonZero<usize>,
    last_partial: Option<FastQRead>,
    last_status: PartialStatus,
    windows_mode: Option<bool>,
    compression_format: niffler::send::compression::Format,
}

impl FastqParser {
    /// # Panics
    /// when rapidgzip & stdin are specified together (validation prevents this)
    pub fn new(
        file: std::fs::File,
        filename: Option<&PathBuf>,
        target_reads_per_block: NonZero<usize>,
        buf_size: usize,
        decompression_options: DecompressionOptions,
    ) -> Result<FastqParser> {
        let (reader, format) = open_decompressed_reader(file, filename, decompression_options)?;

        Ok(FastqParser {
            current_reader: reader,
            current_block: Some(FastQBlock {
                block: Vec::new(),
                entries: Vec::new(),
                first_read_sequential_number: 0,
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
            if *start
                >= self
                    .current_block
                    .as_ref()
                    .expect("current_block must be initialized")
                    .block
                    .len()
            {
                self.current_block
                    .as_mut()
                    .expect("current_block must be initialized")
                    .block
                    .extend(vec![0; self.buf_size]);
            }

            let read = self.current_reader.read(
                &mut self
                    .current_block
                    .as_mut()
                    .expect("current_block must be initialized")
                    .block[*start..],
            )?; // cov:excl-line

            if read == 0 {
                return Ok(false);
            }
            *start += read;
        }
        Ok(true)
    }

    fn next_block(&mut self) -> Result<(FastQBlock, bool)> {
        let mut was_final = false;
        let mut start = self
            .current_block
            .as_ref()
            .expect("current_block must be initialized")
            .block
            .len();
        while self
            .current_block
            .as_ref()
            .expect("current_block must be initialized")
            .entries
            .len()
            < self.target_reads_per_block.into()
        {
            let block_start = start;

            if self.windows_mode.is_none() {
                if !self.advance(&mut start)? {
                    //empty file
                    was_final = true;
                    break;
                }
                while self.windows_mode.is_none() {
                    let block = &self.current_block.as_ref().expect("checked above").block;
                    if memchr::memmem::find(block, b"\r\n").is_some() {
                        self.windows_mode = Some(true);
                        break;
                    } else if memchr::memchr(b'\n', block).is_some() {
                        self.windows_mode = Some(false);
                        break;
                    }
                    //when the bufsize is smaller than the first read name, we need to read more.
                    //pathological? yes.
                    //read until we have at least one newline.
                    if !self.advance(&mut start)? {
                        bail!(
                            "Parsing error: read all of file, but found no newlines. Check your input FASTQ file."
                        );
                    }
                }
            } else if !self.advance(&mut start)? {
                was_final = true;
                break;
            }
            let parse_result = parse_to_fastq_block(
                self.current_block
                    .as_mut()
                    .expect("current_block must be initialized"),
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
        self.current_block
            .as_mut()
            .expect("current_block must be initialized")
            .block
            .resize(start, 0);

        let (mut out_block, new_block) = self
            .current_block
            .take()
            .expect("current_block must be initialized")
            .split_at(self.target_reads_per_block);

        self.current_block = Some(new_block);
        if was_final && let Some(partial) = self.last_partial.take() {
            match self.last_status {
                PartialStatus::InQual => {}
                // cov:excl-start
                PartialStatus::NoPartial => unreachable!(),
                // cov:excl-stop
                _ => bail!("Incomplete final read. Was in state {:?}", self.last_status),
            }
            let final_read = FastQRead::new(partial.name, partial.seq, partial.qual)
                .context("In parsing final read")?;
            out_block.entries.push(final_read);
        }
        Ok((out_block, was_final))
    }
}

impl Parser for FastqParser {
    fn parse(&mut self) -> Result<ParseResult> {
        let (block, was_final) = self.next_block()?;
        Ok(ParseResult {
            output: ParserOutput::Block(block),
            was_final,
        })
    }

    fn bytes_per_base(&self) -> f64 {
        match self.compression_format {
            niffler::send::compression::Format::Gzip
            | niffler::send::compression::Format::Bzip
            | niffler::send::compression::Format::Lzma
            | niffler::send::compression::Format::Zstd => 0.5,
            niffler::send::compression::Format::No => 2.25,
        }
    }
}

/// Columnar FASTQ parser backed by [`parse_pods_from_channel`].
///
/// It owns two background threads: a *reader* thread that pulls (already
/// rapidgzip/niffler-decompressed) bytes off the input in `buffer_size` chunks
/// and feeds them into the pod parser's byte channel, and the *pod parser*
/// thread itself, which emits record-aligned [`PodFastqChunk`]s into
/// `chunk_rx`.
///
/// The pod parser emits one chunk per arbitrary input byte-chunk, so its chunk
/// sizes bear no relation to the requested block size. [`parse_chunk`] therefore
/// re-groups the emitted columns into blocks of exactly `target` reads (the
/// final block may be short), which is what keeps the per-segment input streams
/// in lockstep for the combiner.
///
/// [`parse_chunk`]: PodFastqParser::parse_chunk
pub struct PodFastqParser {
    /// Record-aligned columnar chunks emitted by the pod parser thread.
    chunk_rx: Receiver<PodFastqChunk>,
    /// The pod parser thread; joined at EOF to surface a parse error.
    parser_handle: Option<JoinHandle<Result<()>>>,
    /// The byte-reader thread; joined at EOF to surface an io error.
    reader_handle: Option<JoinHandle<Result<()>>>,
    /// Emitted-but-not-yet-consumed chunks, with `front_consumed` reads already
    /// drained off the front one.
    pending: VecDeque<PodFastqChunk>,
    /// Reads already consumed from `pending.front()`.
    front_consumed: usize,
    /// Reads per emitted block (the molecule-count target).
    target: usize,
    /// Set once `chunk_rx` is closed and drained.
    eof: bool,
    compression_format: niffler::send::compression::Format,
}

impl PodFastqParser {
    /// # Errors
    /// On io errors opening/decompressing the input.
    pub fn new(
        file: std::fs::File,
        filename: Option<&PathBuf>,
        target_reads_per_block: NonZero<usize>,
        buffer_size: usize,
        demux_threads: usize,
        decompression_options: DecompressionOptions,
    ) -> Result<PodFastqParser> {
        let (reader, format) = open_decompressed_reader(file, filename, decompression_options)?;
        let demux_threads = demux_threads.max(1);
        let buffer_size = buffer_size.max(1);

        let (bytes_tx, bytes_rx) = channel::bounded::<Arc<Vec<u8>>>(demux_threads * 4);
        let (chunk_tx, chunk_rx) = channel::bounded::<PodFastqChunk>(demux_threads * 4);

        // Reader thread — fill `buffer_size` chunks and feed the byte channel.
        let reader_handle = std::thread::spawn(move || -> Result<()> {
            let mut reader = reader;
            loop {
                let mut buf = vec![0u8; buffer_size];
                let mut filled = 0;
                while filled < buffer_size {
                    let n = reader.read(&mut buf[filled..])?; // cov:excl-line
                    if n == 0 {
                        break;
                    }
                    filled += n;
                }
                if filled == 0 {
                    break;
                }
                buf.truncate(filled);
                if bytes_tx.send(Arc::new(buf)).is_err() {
                    // Pod parser hung up (an error downstream); stop feeding it.
                    break; // cov:excl-line
                }
                if filled < buffer_size {
                    break; // short read ⇒ EOF
                }
            }
            Ok(())
        });

        let parser_handle =
            std::thread::spawn(move || parse_pods_from_channel(bytes_rx, chunk_tx, demux_threads, None));

        Ok(PodFastqParser {
            chunk_rx,
            parser_handle: Some(parser_handle),
            reader_handle: Some(reader_handle),
            pending: VecDeque::new(),
            front_consumed: 0,
            target: target_reads_per_block.get(),
            eof: false,
            compression_format: format,
        })
    }

    /// Join the background threads, surfacing the first error. Called once the
    /// chunk channel has closed (`eof`), at which point both threads have run to
    /// completion: the pod parser dropped `chunk_tx`, and the reader stops as
    /// soon as the pod parser drops `bytes_rx`.
    fn finish_threads(&mut self) -> Result<()> {
        if let Some(handle) = self.parser_handle.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("pod fastq parser thread panicked"))??;
        }
        if let Some(handle) = self.reader_handle.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("pod fastq reader thread panicked"))??;
        }
        Ok(())
    }

    /// Produce the next block of up to `target` reads as a [`FastQChunk`],
    /// together with `was_final`. Re-groups the pod parser's arbitrarily-sized
    /// emissions into exact block sizes by copying reads through fresh column
    /// builders; the leftover tail of a straddling emission is carried in
    /// `pending` / `front_consumed` for the next call.
    fn next_chunk(&mut self) -> Result<(FastQChunk, bool)> {
        let mut names = StringPodBuilder::with_capacity(0, self.target);
        let mut seq_quals = DualStringPodBuilder::with_capacity(0, self.target);
        let mut count = 0usize;

        while count < self.target {
            let Some(chunk) = self.pending.front() else {
                match self.chunk_rx.recv() {
                    Ok(chunk) => {
                        // The pod parser never emits empty chunks, but guard anyway.
                        if chunk.names.len() > 0 {
                            self.pending.push_back(chunk);
                        }
                    }
                    Err(_) => {
                        self.eof = true;
                        break;
                    }
                }
                continue;
            };

            let available = chunk.names.len() - self.front_consumed;
            let take = available.min(self.target - count);
            let start = self.front_consumed;
            for i in start..start + take {
                names.push(chunk.names.get(i).as_bytes());
                let (seq, qual) = chunk.reads.pair(i);
                seq_quals.push(seq.as_bytes(), qual.as_bytes());
            }
            self.front_consumed += take;
            count += take;
            if self.front_consumed >= chunk.names.len() {
                self.pending.pop_front();
                self.front_consumed = 0;
            }
        }

        if self.eof {
            self.finish_threads()?;
        }

        let pluses =
            StringPod::new_all_empty(u32::try_from(count).expect("too many reads in a block for u32"));
        let chunk = FastQChunk {
            names: names.finish(),
            seq_quals: seq_quals.finish(),
            pluses,
        };
        let was_final = self.eof && self.pending.is_empty();
        Ok((chunk, was_final))
    }
}

impl Parser for PodFastqParser {
    /// Emits columnar [`FastQChunk`] blocks of exactly `target` reads (the final
    /// block may be short).
    ///
    /// # Panics
    /// If a single block would exceed `u32::MAX` reads.
    fn parse(&mut self) -> Result<ParseResult> {
        let (chunk, was_final) = self.next_chunk()?;
        Ok(ParseResult {
            output: ParserOutput::Chunk(chunk),
            was_final,
        })
    }

    fn bytes_per_base(&self) -> f64 {
        match self.compression_format {
            niffler::send::compression::Format::Gzip
            | niffler::send::compression::Format::Bzip
            | niffler::send::compression::Format::Lzma
            | niffler::send::compression::Format::Zstd => 0.5,
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
    InSpacerExpectPlus,
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

#[expect(
    clippy::too_many_lines,
    reason = "it is a large state machine, therefore many lines"
)]
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
                last_status = PartialStatus::InSpacerExpectPlus;
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
        let last_read2 = last_read
            .as_mut()
            .expect("last_read must be Some in this code path");
        let next_newline = newline_iterator.next();
        // debug!("Continue reading inname Next_newline: {next_newline:?}");

        if let Some(next_newline) = next_newline {
            match &mut last_read2.name {
                FastQElement::Owned(name) => {
                    name.extend_from_slice(&input[pos..start_offset + next_newline]);
                }
                // cov:excl-start
                FastQElement::Local(_) => panic!("Should not happen"),
                // cov:excl-stop
            }
            pos = start_offset + next_newline + newline_length;
            last_status = PartialStatus::InSeq;
        } else {
            let (status, name_end) = if windows_mode && input[stop - 1] == b'\r' {
                (PartialStatus::InNameNewline, stop - 1)
            } else {
                (PartialStatus::InName, stop)
            };

            match &mut last_read2.name {
                FastQElement::Owned(name) => {
                    name.extend_from_slice(&input[pos..name_end]);
                }
                // cov:excl-start
                FastQElement::Local(_) => panic!("Should not happen"),
                // cov:excl-stop
            }
            // debug!("Returning in name 1 {:?}", last_read.as_ref().unwrap());
            return Ok(FastQBlockParseResult {
                status,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
        }
        // debug!( "Continue reading name: {next_newline} {} {}", input.len(), std::str::from_utf8(&input[..next_newline]).unwrap());
    }
    if PartialStatus::InSeq == last_status {
        let last_read2 = last_read
            .as_mut()
            .expect("last_read must be Some in this code path");
        let next_newline = newline_iterator.next();
        // debug!("Continue reading inseq Next_newline: {next_newline:?}");
        if let Some(next_newline) = next_newline {
            match &mut last_read2.seq {
                FastQElement::Owned(seq) => {
                    seq.extend_from_slice(&input[pos..start_offset + next_newline]);
                }
                // cov:excl-start
                FastQElement::Local(_) => unreachable!(),
                // cov:excl-stop
            }
            pos = start_offset + next_newline + newline_length;
        } else {
            let (status, seq_end) = if windows_mode && input[stop - 1] == b'\r' {
                (PartialStatus::InSeqNewline, stop - 1)
            } else {
                (PartialStatus::InSeq, stop)
            };

            match &mut last_read2.seq {
                FastQElement::Owned(seq) => {
                    seq.extend_from_slice(&input[pos..seq_end]);
                }
                // cov:excl-start
                FastQElement::Local(_) => panic!("Should not happen"),
                // cov:excl-stop
            }
            // debug!("Returning in seq1: {:?}", last_read.as_ref().unwrap());
            return Ok(FastQBlockParseResult {
                status,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
        }
        if pos < stop && input[pos] != b'+' {
            bail!(
                "(partial) Expected + after sequence in input. Position {pos}, was {}, Read name was: '{}'.\nIf your Fastq is line-wrapped, sorry that's not supported.",
                input[pos],
                BString::from(last_read2.name.get(input))
            );
        }
        if pos < stop {
            last_status = PartialStatus::InSpacer;
        } else {
            return Ok(FastQBlockParseResult {
                status: PartialStatus::InSpacerExpectPlus,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
        }
    }

    if PartialStatus::InSpacerExpectPlus == last_status {
        if pos < stop {
            if input[pos] != b'+' {
                bail!(
                    "(spacer) Expected + after sequence in input. Position {pos}, was {}, Read name was: '{}'.\nIf your Fastq is line-wrapped, sorry that's not supported.",
                    input[pos],
                    BString::from(
                        last_read
                            .expect("last read must have been set")
                            .name
                            .get(input)
                    )
                );
            }
            last_status = PartialStatus::InSpacer;
        } else {
            //read more bytes please. But we were already reading more bytes?
            //maybe a bug in the decompression, handing in empty blocks?
            //I don't think we can trigger this in normal operation,
            //but if we do, we just continue with the next block
            // cov:excl-start
            return Ok(FastQBlockParseResult {
                status: PartialStatus::InSpacerExpectPlus,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
            // cov:excl-stop
        }
    }

    if PartialStatus::InSpacer == last_status {
        let next_newline = newline_iterator.next();
        if let Some(next_newline) = next_newline {
            // println!(
            //     "Continue reading spacer: {next_newline} {} '{}'",
            //     input.len(),
            //     std::str::from_utf8(&input[pos..pos + next_newline]).unwrap()
            // );
            pos = start_offset + next_newline + newline_length;
        } else {
            let status = if windows_mode && input[stop - 1] == b'\r' {
                PartialStatus::InSpacerNewline
            } else {
                PartialStatus::InSpacer
            };

            // println!("Returning in spacer");
            return Ok(FastQBlockParseResult {
                status,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
        }

        last_status = PartialStatus::InQual;
    }
    if PartialStatus::InQual == last_status {
        let last_read2 = last_read
            .as_mut()
            .expect("last_read must be Some in this code path");
        let next_newline = newline_iterator.next();
        if let Some(next_newline) = next_newline {
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
                // cov:excl-start
                FastQElement::Local(_) => panic!("Should not happen"),
                // cov:excl-stop
            }
            pos = start_offset + next_newline + newline_length;
        } else {
            let (status, qual_end) = if windows_mode && input[stop - 1] == b'\r' {
                (PartialStatus::InQualNewline, stop - 1)
            } else {
                (PartialStatus::InQual, stop)
            };

            match &mut last_read2.qual {
                FastQElement::Owned(qual) => {
                    qual.extend_from_slice(&input[pos..qual_end]);
                }
                // cov:excl-start
                FastQElement::Local(_) => panic!("Should not happen"),
                // cov:excl-stop
            }
            return Ok(FastQBlockParseResult {
                status,
                partial_read: Some(last_read.expect("last_read must be Some")),
                windows_mode,
            });
        }
    }
    if let Some(last_read) = last_read {
        last_read.verify().with_context(|| {
            // cov:excl-start
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
        // cov:excl-stop

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
                    "Unexpected symbol where @ was expected in input. Position {}, was '{}' (0x{:x}). Check your input FASTQ",
                    pos,
                    letter,
                    input[pos]
                );
            }
        }
        let end_of_name = newline_iterator.next();
        let (name_start, name_end) = {
            if let Some(end_of_name) = end_of_name {
                let r = (pos + 1, end_of_name + start_offset);
                if r.0 >= r.1 {
                    bail!("Empty name in input FASTQ. Verify your input files are proper FASTQ.");
                }
                pos = start_offset + end_of_name + newline_length;
                r
            } else {
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
                    .expect("FastQRead creation should not fail for partial read"),
                );
                break;
            }
        };
        let end_of_seq = newline_iterator.next();
        let (seq_start, seq_end) = {
            if let Some(end_of_seq) = end_of_seq {
                let r = (pos, end_of_seq + start_offset);
                pos = start_offset + end_of_seq + newline_length;
                r
            } else {
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
                "Expected + after sequence in input. Position {pos}, was {}, Read name was: '{}'.\nIf your Fastq is line-wrapped, sorry that's not supported.",
                input[pos],
                pos
            );
        }
        let end_of_spacer = newline_iterator.next();

        if let Some(end_of_spacer) = end_of_spacer {
            pos = start_offset + end_of_spacer + newline_length;
        } else {
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

        let end_of_qual = newline_iterator.next();
        let (qual_start, qual_end) = {
            if let Some(end_of_qual) = end_of_qual {
                let r = (pos, end_of_qual + start_offset);
                pos = start_offset + end_of_qual + newline_length;
                r
            } else {
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
