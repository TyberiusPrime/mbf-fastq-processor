use anyhow::{Context, Result, bail};
use std::num::NonZero;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::{io::Read, path::PathBuf};

use crossbeam::channel::{self, Receiver};
use stringpod::StringPod;

use crate::CompressionFormat;
use crate::blocks::FastQChunk;
use crate::io::input::{DecompressionOptions, open_decompressed_reader};
use crate::io::parsers::{ParseResult, Parser};
use crate::io::pod_parser::{Chunk, FastqChunk as PodFastqChunk, parse_pods_from_channel};
#[cfg(unix)]
use crate::io::{
    input::shm_eligible_format,
    parsers::shm::{ShmChunkReader, shm_enabled, spawn_shm_chunk_reader},
};

/// Columnar FASTQ parser backed by [`parse_pods_from_channel`].
///
/// It owns two background threads: a *reader* thread that pulls (already
/// subprocess-decompressed) bytes off the input in `buffer_size` chunks
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
    compression_format: crate::CompressionFormat,
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

        // Shared-memory fast path: out-of-process-decoded gzip *or* zstd FASTQ,
        // unless the `FASTQRAB_DECOMP_SHM=0` escape hatch is set. The decompressor
        // memcpies each chunk into a shared slot and we parse it in place — no
        // bulk pipe copies. Other codecs, stdin/FIFO, and the Default path stay on
        // the existing `Read`-based reader below.
        #[cfg(unix)]
        if let Some(path) = filename
            && shm_enabled()
            && let Some(format) = shm_eligible_format(&file)
        {
            return Self::new_shm(
                path,
                format,
                decompression_options.thread_count,
                demux_threads,
            );
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
                    let n = reader.read(&mut buf[filled..])?;
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
                    break; //cov:ignore-line
                }
                if filled < buffer_size {
                    break; // short read ⇒ EOF
                }
            }
            Ok(())
        });

        let parser_handle =
            std::thread::spawn(move || parse_pods_from_channel(bytes_rx, chunk_tx, demux_threads));

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

    /// Shared-memory constructor (Unix, gzip/zstd FASTQ). Stands up the shared
    /// [`ShmChunkReader`] transport (descriptor-reader + slot-return threads, the
    /// mapped region) and spawns the pod *parser* thread consuming its borrowed
    /// chunks. The region is kept alive for the parser's lifetime so every
    /// borrowed chunk stays valid.
    #[cfg(unix)]
    fn new_shm(
        path: &std::path::Path,
        format: crate::io::input::DecompressorFormat,
        thread_count: crate::io::parsers::ThreadCount,
        demux_threads: usize,
    ) -> Result<PodFastqParser> {
        let ShmChunkReader {
            bytes_rx,
            reader_handle,
            slot_writer_handle,
            child,
            region,
        } = spawn_shm_chunk_reader(path, format, thread_count, demux_threads)?;

        let (chunk_tx, chunk_rx) = channel::bounded::<PodFastqChunk>(demux_threads * 4); //mutants::skip
        let parser_handle =
            std::thread::spawn(move || parse_pods_from_channel(bytes_rx, chunk_tx, demux_threads));

        Ok(PodFastqParser {
            chunk_rx,
            parser_handle: Some(parser_handle),
            reader_handle: Some(reader_handle),
            slot_writer_handle: Some(slot_writer_handle),
            child: Some(child),
            _region: Some(region as Arc<dyn std::any::Any + Send + Sync>),
            peeked: None,
            eof: false,
            compression_format: format.to_compression(),
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
                .map_err(|e| anyhow::anyhow!("pod fastq parser thread panicked: {e:?}"))??;
        } // cov:excl-line
        if let Some(handle) = self.reader_handle.take() {
            handle
                .join()
                .map_err(|e| anyhow::anyhow!("pod fastq reader thread panicked: {e:?}"))??;
        } // cov:excl-line
        // shm mode: the parser + reader are done, so every borrowed chunk has
        // dropped and returned its slot id; the slot-return channel is now closed
        // and the writer thread can finish relaying.
        if let Some(handle) = self.slot_writer_handle.take() {
            handle
                .join()
                .map_err(|e| anyhow::anyhow!("pod fastq slot-return thread panicked: {e:?}"))?;
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
                bail!("fastqrab-decompressor exited unsuccessfully: {status}"); //cov:ignore-line 
                //it bounces coveraged non-covered?
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
                Ok(chunk) if !chunk.names.is_empty() => return Some(chunk), //mutants::skip
                Ok(_) => {}                                                 //cov:excl-line
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
            output: chunk,
            was_final,
        })
    }

    fn bytes_per_base(&self) -> f64 {
        match self.compression_format {
            CompressionFormat::Gzip | CompressionFormat::Zstd => 0.5,
            CompressionFormat::Uncompressed => 2.25,
        }
    }
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
            DecompressionOptions {
                thread_count: crate::io::parsers::ThreadCount(std::num::NonZero::<usize>::MIN),
            },
        )
        .expect("parser");

        let mut out: Reads = Vec::new();
        let mut sizes = Vec::new();
        loop {
            let res = parser.parse().expect("parse");
            let chunk = res.output;
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
        assert_eq!(
            sizes.iter().sum::<usize>(),
            total,
            "all reads accounted for"
        );
    }

    #[expect(clippy::string_slice, reason = "No utf-8 issues in this test data")]
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
