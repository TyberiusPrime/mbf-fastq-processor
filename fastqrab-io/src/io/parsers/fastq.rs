use anyhow::{Context, Result, bail};
use bstr::BString;
use niffler;
use std::num::NonZero;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{io::Read, path::PathBuf};

use crossbeam::channel::{self, Receiver};
use stringpod::StringPod;

use crate::blocks::FastQChunk;
use crate::io::parsers::{ParseResult, Parser, ParserOutput};
use crate::io::pod_parser::{Chunk, FastqChunk as PodFastqChunk, parse_pods_from_channel};
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
/// sizes bear no relation to the requested block size. Those native chunks are
/// forwarded straight through (no regroup, no extra copy); the combiner aligns
/// per-segment block sizes by slicing, so a fixed emitted size is no longer
/// required to keep the segment streams in lockstep.
pub struct PodFastqParser {
    /// Record-aligned columnar chunks emitted by the pod parser thread.
    chunk_rx: Receiver<PodFastqChunk>,
    /// The pod parser thread; joined at EOF to surface a parse error.
    parser_handle: Option<JoinHandle<Result<()>>>,
    /// The byte-reader thread; joined at EOF to surface an io error. In pipe mode
    /// this reads decompressed bytes; in shm mode it reads `(slot, len)`
    /// descriptors and wraps each slot as a borrowed [`Chunk`].
    reader_handle: Option<JoinHandle<Result<()>>>,
    /// shm mode only: relays freed slot ids to the decompressor's stdin.
    slot_writer_handle: Option<JoinHandle<()>>,
    /// shm mode only: the decompressor child, waited on at EOF to surface a
    /// non-zero exit (e.g. a mid-stream decode error after a truncated stream).
    child: Option<std::process::Child>,
    /// shm mode only: keeps the shared-memory mapping alive (type-erased so the
    /// field is cross-platform) for as long as any borrowed [`Chunk`] could be.
    _region: Option<Arc<dyn std::any::Any + Send + Sync>>,
    /// One-chunk lookahead. The pod parser's native chunks are forwarded
    /// straight through (no regroup); we peek one chunk ahead so the *last*
    /// data block can carry `was_final` instead of trailing an empty block —
    /// the shape the chained parser expects. `None` until the first `parse`.
    peeked: Option<PodFastqChunk>,
    /// Set once `chunk_rx` is closed and drained.
    eof: bool,
    compression_format: niffler::send::compression::Format,
}

/// Whether the shared-memory decompressor transport is enabled. On by default;
/// `FASTQRAB_DECOMP_SHM=0` forces the legacy pipe path (A/B and field escape
/// hatch).
#[cfg(unix)]
fn shm_enabled() -> bool {
    !matches!(std::env::var("FASTQRAB_DECOMP_SHM").as_deref(), Ok("0"))
}

/// Shared-memory slot size in bytes (`FASTQRAB_DECOMP_SHM_SLOT_SIZE`, default
/// 8 MiB — comfortably above the decoder's ~4 MiB chunk so a chunk usually fits
/// one slot). Tunable; tests shrink it to force the multi-slot chunk-split path.
#[cfg(unix)]
fn shm_slot_size() -> usize {
    std::env::var("FASTQRAB_DECOMP_SHM_SLOT_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(8 * 1024 * 1024)
}

/// Number of shared-memory slots (`FASTQRAB_DECOMP_SHM_SLOTS`, default `fallback`
/// computed from the thread counts). Tunable; tests shrink it to a tiny ring to
/// stress backpressure / recycling. The pipeline is deadlock-free for any ring
/// size ≥ 1 (demux workers never wait on a slot to finish).
#[cfg(unix)]
fn shm_slot_count(fallback: usize) -> usize {
    std::env::var("FASTQRAB_DECOMP_SHM_SLOTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(fallback)
}

/// True only for a *regular* file whose first two bytes are the gzip magic
/// (`1f 8b`). Peeks and rewinds so, when this returns false, the fall-through
/// `Read`-based path sees the file untouched at offset 0. Pipes/FIFOs (not
/// seekable) and non-gzip inputs return false and keep the existing transport.
#[cfg(unix)]
fn shm_eligible_gzip(file: &std::fs::File) -> bool {
    use std::io::{Read as _, Seek as _, SeekFrom};
    let Ok(meta) = file.metadata() else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    let mut handle: &std::fs::File = file;
    let mut magic = [0u8; 2];
    let read = handle.read_exact(&mut magic);
    // Always rewind a regular file, even on a short read, so the fall-through
    // reader is unaffected.
    let _ = handle.seek(SeekFrom::Start(0));
    read.is_ok() && magic == [0x1f, 0x8b]
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
        // The emitted block size is now the pod parser's native per-decode-chunk
        // size (the combiner aligns segments by slicing), so the requested
        // reads-per-block is retained only as an API hint and otherwise ignored.
        let _ = target_reads_per_block;
        let demux_threads = demux_threads.max(1);
        let buffer_size = buffer_size.max(1);

        // Shared-memory fast path: rapidgzip-decoded gzip FASTQ, unless the
        // `FASTQRAB_DECOMP_SHM=0` escape hatch is set. The decompressor memcpies
        // each chunk into a shared slot and we parse it in place — no bulk pipe
        // copies. FASTA, non-gzip, stdin/FIFO, and the Default path stay on the
        // existing `Read`-based reader below.
        #[cfg(unix)]
        if let DecompressionOptions::Rapidgzip { thread_count } = decompression_options
            && let Some(path) = filename
            && shm_enabled()
            && shm_eligible_gzip(&file)
        {
            return Self::new_shm(path, thread_count, demux_threads);
        }

        let (reader, format) = open_decompressed_reader(file, filename, decompression_options)?;

        let (bytes_tx, bytes_rx) = channel::bounded::<Chunk>(demux_threads * 4);
        let (chunk_tx, chunk_rx) = channel::bounded::<PodFastqChunk>(demux_threads * 4);
        // Recycle channel: demux workers hand back each drained input buffer
        // (see `parse_pods_from_channel`) so the reader can refill it instead of
        // allocating. Reusing the allocation keeps its pages faulted-in, which is
        // what removes the `alloc_vec_from_elem` cost and the minor-fault churn.
        let (recycle_tx, recycle_rx) = channel::bounded::<Vec<u8>>(demux_threads * 4);

        // Reader thread — fill `buffer_size` chunks and feed the byte channel.
        let reader_handle = std::thread::spawn(move || -> Result<()> {
            let mut reader = reader;
            loop {
                // Reuse a recycled buffer when one is available; otherwise start
                // fresh. A recycled buffer was filled to `buffer_size` on a prior
                // pass (we only ever shrink it via `truncate`), so its first
                // `buffer_size` bytes are already initialized and we can re-expose
                // them without a memset — `read` overwrites the prefix we consume
                // and we `truncate` to what was actually read. A fresh buffer
                // (cold start / empty recycle channel) still has to be zeroed.
                let mut buf = recycle_rx.try_recv().unwrap_or_default();
                if buf.capacity() >= buffer_size {
                    // SAFETY: `buf` came back from a previous full-size pass, so
                    // bytes `0..buffer_size` are initialized (plain `u8`, no
                    // `Drop`); nothing reads them before `read` overwrites the
                    // consumed prefix.
                    unsafe { buf.set_len(buffer_size) };
                } else {
                    buf.resize(buffer_size, 0);
                }
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
                if bytes_tx
                    .send(Chunk::owned(Arc::new(buf), Some(recycle_tx.clone())))
                    .is_err()
                {
                    // Pod parser hung up (an error downstream); stop feeding it.
                    break; // cov:excl-line
                }
                if filled < buffer_size {
                    break; // short read ⇒ EOF
                }
            }
            Ok(())
        });

        let parser_handle = std::thread::spawn(move || {
            parse_pods_from_channel(bytes_rx, chunk_tx, demux_threads)
        });

        Ok(PodFastqParser {
            chunk_rx,
            parser_handle: Some(parser_handle),
            reader_handle: Some(reader_handle),
            slot_writer_handle: None,
            child: None,
            _region: None,
            peeked: None,
            eof: false,
            compression_format: format,
        })
    }

    /// Shared-memory constructor (Unix, gzip FASTQ via rapidgzip). Spawns the
    /// decompressor in shm mode and stands up three threads: a *descriptor
    /// reader* that wraps each `(slot, len)` as a borrowed [`Chunk`] and feeds
    /// the pod parser, a *slot-return writer* that relays freed slot ids back to
    /// the child, and the pod *parser* itself. The shared mapping is kept alive
    /// for the parser's lifetime so every borrowed chunk stays valid.
    #[cfg(unix)]
    fn new_shm(
        path: &std::path::Path,
        thread_count: crate::io::parsers::ThreadCount,
        demux_threads: usize,
    ) -> Result<PodFastqParser> {
        use crate::io::input::{ShmRapidgzip, spawn_rapidgzip_shm};
        use std::io::{Read as _, Write as _};

        // Slots are sized a bit larger than the decoder's ~4 MiB chunk so the
        // common chunk lands in a single slot (an oversized chunk still splits).
        // The region is `memfd`-backed and sparse, so unused slots cost no
        // physical memory — only in-flight slots fault in — and we can size the
        // ring generously for overlap. Both are overridable
        // (`FASTQRAB_DECOMP_SHM_SLOT_SIZE` / `_SLOTS`) for tuning and to stress
        // the small-ring / multi-slot-split paths under test.
        let slot_size = shm_slot_size();
        let depth = thread_count.0.get().max(demux_threads);
        let slots = shm_slot_count((depth * 2 + 4).clamp(8, 64));

        let ShmRapidgzip {
            region,
            mut descriptors,
            mut slot_return,
            child,
            slots,
            slot_size,
        } = spawn_rapidgzip_shm(path, thread_count, slots, slot_size)?;

        // `bytes_tx` capacity ≥ `slots` guarantees the descriptor reader can
        // always enqueue an in-flight chunk without blocking, so the demux
        // workers (which never wait on a slot to finish) keep draining and
        // returning slots — no deadlock for any ring size ≥ 1.
        let (bytes_tx, bytes_rx) = channel::bounded::<Chunk>(slots);
        let (chunk_tx, chunk_rx) = channel::bounded::<PodFastqChunk>(demux_threads * 4);
        // Slot-return channel: a `Chunk`'s drop pushes its freed slot id here;
        // the writer thread relays them to the child's stdin.
        let (slot_ret_tx, slot_ret_rx) = channel::unbounded::<u32>();

        let region_for_reader = Arc::clone(&region);
        let reader_handle = std::thread::spawn(move || -> Result<()> {
            let mut desc = [0u8; 8];
            loop {
                if let Err(e) = descriptors.read_exact(&mut desc) {
                    // EOF before the sentinel ⇒ the decompressor died; surface it
                    // (its stderr is inherited, so the real cause is already shown).
                    return Err(anyhow::Error::new(e).context(
                        "fastqrab-decompressor closed before sending its EOF sentinel",
                    ));
                }
                let slot = u32::from_le_bytes(desc[0..4].try_into().expect("4 bytes"));
                let len = u32::from_le_bytes(desc[4..8].try_into().expect("4 bytes")) as usize;
                if slot == u32::MAX {
                    break; // EOF sentinel
                }
                // SAFETY: the decompressor only emits `slot < slots` and
                // `len <= slot_size`, and won't reuse the slot until we return its
                // id (on chunk drop), so this borrow into the mapped region is
                // valid and unaliased for the chunk's lifetime.
                let ptr = unsafe { region_for_reader.as_ptr().add(slot as usize * slot_size) };
                // SAFETY: as above — `ptr..ptr+len` is inside the live mapping
                // (held by `region_for_reader`) and the slot is single-owner
                // until this chunk drops and returns it.
                let chunk = unsafe { Chunk::shared(ptr, len, slot, slot_ret_tx.clone()) };
                if bytes_tx.send(chunk).is_err() {
                    break; // pod parser hung up (a downstream error)
                }
            }
            Ok(())
        });

        let slot_writer_handle = std::thread::spawn(move || {
            for slot in slot_ret_rx {
                // Ignore write errors: the child may have already exited after the
                // sentinel, so its stdin is closed (EPIPE) — that's expected.
                if slot_return.write_all(&slot.to_le_bytes()).is_err() {
                    break;
                }
            }
            let _ = slot_return.flush();
        });

        let parser_handle =
            std::thread::spawn(move || parse_pods_from_channel(bytes_rx, chunk_tx, demux_threads));

        Ok(PodFastqParser {
            chunk_rx,
            parser_handle: Some(parser_handle),
            reader_handle: Some(reader_handle),
            slot_writer_handle: Some(slot_writer_handle),
            child: Some(child),
            _region: Some(region),
            peeked: None,
            eof: false,
            compression_format: niffler::send::compression::Format::Gzip,
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
        // shm mode: the parser + reader are done, so every borrowed chunk has
        // dropped and returned its slot id; the slot-return channel is now closed
        // and the writer thread can finish relaying.
        if let Some(handle) = self.slot_writer_handle.take() {
            handle
                .join()
                .map_err(|_| anyhow::anyhow!("pod fastq slot-return thread panicked"))?;
        }
        // shm mode: surface a non-zero decompressor exit. This catches the case
        // where a mid-stream decode error makes the child emit its EOF sentinel
        // (its input channel closed) yet exit non-zero — the data we parsed would
        // be silently truncated otherwise.
        if let Some(mut child) = self.child.take() {
            let status = child
                .wait()
                .context("waiting on fastqrab-decompressor (shm mode)")?;
            if !status.success() {
                bail!("fastqrab-decompressor exited unsuccessfully: {status}");
            }
        }
        Ok(())
    }

    /// Pull the next non-empty chunk (from the one-chunk lookahead or the
    /// channel), skipping any empty chunks. `None` once the channel is closed
    /// and drained.
    fn recv_nonempty(&mut self) -> Option<PodFastqChunk> {
        if let Some(chunk) = self.peeked.take() {
            return Some(chunk);
        }
        loop {
            match self.chunk_rx.recv() {
                // The pod parser never emits empty chunks, but guard anyway.
                Ok(chunk) if chunk.names.len() > 0 => return Some(chunk),
                Ok(_) => {}
                Err(_) => return None,
            }
        }
    }

    /// Produce the next block as a [`FastQChunk`], together with `was_final`.
    ///
    /// The pod parser's native per-decode-chunk emission is forwarded straight
    /// through — its columns are moved in with no regroup/copy — so blocks are
    /// roughly one decode chunk of reads each. The combiner aligns segment block
    /// sizes by slicing, so the emitted size no longer needs to match a fixed
    /// `block_size`. A one-chunk lookahead lets the final data block carry
    /// `was_final` rather than trailing a separate empty block.
    fn next_chunk(&mut self) -> Result<(FastQChunk, bool)> {
        let Some(chunk) = self.recv_nonempty() else {
            // Channel drained: emit the terminal empty block.
            self.eof = true;
            self.finish_threads()?;
            return Ok((FastQChunk::new_empty(), true));
        };

        // Peek one chunk ahead: if there's none, this chunk is the last.
        let was_final = match self.recv_nonempty() {
            Some(next) => {
                self.peeked = Some(next);
                false
            }
            None => {
                self.eof = true;
                self.finish_threads()?;
                true
            }
        };

        let count = chunk.names.len();
        let pluses = StringPod::new_all_empty(
            u32::try_from(count).expect("too many reads in a block for u32"),
        );
        let out = FastQChunk {
            names: chunk.names,
            seq_quals: chunk.reads,
            pluses,
        };
        Ok((out, was_final))
    }
}

impl Parser for PodFastqParser {
    /// Emits the pod parser's native columnar [`FastQChunk`] blocks (roughly one
    /// decode chunk of reads each), terminated by an empty final block.
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

#[cfg(test)]
mod pod_regroup_tests {
    //! End-to-end coverage of [`PodFastqParser`] forwarding the pod parser's
    //! native per-decode-chunk blocks straight through (the regroup was deleted;
    //! the combiner now aligns sizes by slicing). We drive the real parser
    //! through a temp file at several `(target, buffer_size)` ratios and assert
    //! the recovered reads are identical and complete regardless of `target` —
    //! `target` no longer governs the emitted block sizes.
    use super::*;
    use crate::io::input::DecompressionOptions;
    use std::io::Write as _;

    type Reads = Vec<(String, String, String)>;

    fn make_payload(reads: &[(String, String, String)]) -> Vec<u8> {
        let mut v = Vec::new();
        for (name, seq, qual) in reads {
            v.push(b'@');
            v.extend_from_slice(name.as_bytes());
            v.push(b'\n');
            v.extend_from_slice(seq.as_bytes());
            v.extend_from_slice(b"\n+\n");
            v.extend_from_slice(qual.as_bytes());
            v.push(b'\n');
        }
        v
    }

    /// Run the parser to completion, returning the recovered reads and the
    /// per-block read counts.
    fn run(payload: &[u8], target: usize, buffer_size: usize) -> (Reads, Vec<usize>) {
        let mut tmp = tempfile::NamedTempFile::new().expect("tempfile");
        tmp.write_all(payload).expect("write");
        tmp.flush().expect("flush");
        let file = tmp.reopen().expect("reopen");

        let mut parser = PodFastqParser::new(
            file,
            None,
            NonZero::new(target).expect("nonzero target"),
            buffer_size,
            2,
            DecompressionOptions::Default,
        )
        .expect("parser");

        let mut out: Reads = Vec::new();
        let mut sizes = Vec::new();
        loop {
            let res = parser.parse().expect("parse");
            let ParserOutput::Chunk(chunk) = res.output else {
                panic!("PodFastqParser must emit columnar chunks");
            };
            sizes.push(chunk.names.len());
            for i in 0..chunk.names.len() {
                let name = chunk.names.get(i).to_string();
                let (seq, qual) = chunk.seq_quals.pair(i);
                out.push((name, seq.to_string(), qual.to_string()));
            }
            if res.was_final {
                break;
            }
        }
        (out, sizes)
    }

    /// Blocks are now native-sized, so we only assert completeness: every read
    /// is accounted for, and no non-final block is empty (the parser skips empty
    /// emissions; a trailing `0` is the terminal block).
    fn assert_block_sizes(sizes: &[usize], total: usize) {
        for (i, &s) in sizes.iter().enumerate() {
            if i + 1 < sizes.len() {
                assert!(s > 0, "non-final block {i} should be non-empty");
            }
        }
        assert_eq!(sizes.iter().sum::<usize>(), total, "all reads accounted for");
    }

    fn variable_reads(n: usize) -> Reads {
        (0..n)
            .map(|i| {
                let len = 1 + (i % 11);
                (
                    format!("read.{i} some comment"),
                    "ACGT".repeat(len)[..len].to_string(),
                    "IIII".repeat(len)[..len].to_string(),
                )
            })
            .collect()
    }

    fn fixed_reads(n: usize, len: usize) -> Reads {
        (0..n)
            .map(|i| {
                (
                    format!("rd{i:08}"), // fixed-width names too
                    "A".repeat(len),
                    "F".repeat(len),
                )
            })
            .collect()
    }

    #[test]
    fn regroups_variable_reads_across_ratios() {
        let reads = variable_reads(137);
        let payload = make_payload(&reads);
        for &target in &[1usize, 7, 50, 137, 500] {
            for &buf in &[8usize, 64, 4096] {
                let (got, sizes) = run(&payload, target, buf);
                assert_eq!(got, reads, "target={target} buf={buf}");
                assert_block_sizes(&sizes, reads.len());
            }
        }
    }

    #[test]
    fn regroups_fixed_reads_across_ratios() {
        let reads = fixed_reads(200, 32);
        let payload = make_payload(&reads);
        for &target in &[1usize, 33, 200, 1000] {
            for &buf in &[16usize, 256, 8192] {
                let (got, sizes) = run(&payload, target, buf);
                assert_eq!(got, reads, "target={target} buf={buf}");
                assert_block_sizes(&sizes, reads.len());
            }
        }
    }
}
