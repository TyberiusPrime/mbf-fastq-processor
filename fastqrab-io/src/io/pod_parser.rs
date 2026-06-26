//! Channel-driven columnar FASTQ parser producing [`StringPod`] columns.
//!
//! This is source-agnostic: it consumes a channel of *already-decompressed*
//! byte chunks (`Arc<Vec<u8>>`) — from a plain file, a gzip/bgzf/zstd
//! decompressor, a socket, anything — and emits one [`FastqChunk`] per input
//! chunk, each chunk record-aligned. The chunking is arbitrary: decode-chunk
//! boundaries never align to records, and the parser threads a one-line carry
//! across them so records (and even individual record lines) may straddle any
//! number of chunk boundaries.
//!
//! It was extracted from the `rusty-rapidgzip` FASTQ path: there the byte
//! channel was fed by the parallel gzip decoder; here the producer is whoever
//! calls [`parse_pods_from_channel`]. The split/record-alignment machinery is
//! unchanged — a parallel "demux" pool buckets each chunk's lines by
//! `index mod 4` without knowing the absolute phase, and a cheap serial
//! collector rotates the buckets onto (name, seq, `+`, qual), strips the `@`,
//! validates, and emits.

use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use crossbeam::channel::{self, Receiver, Sender};

use stringpod::{DualStringPod, StringPod, StringPodBuilder};

/// A decompressed byte chunk fed to the pod parser, abstracted over its backing
/// storage so the same demux pipeline drives both the in-process pipe reader and
/// the out-of-process shared-memory transport.
///
/// It derefs to `&[u8]` (the only thing the demux scan needs) and *releases its
/// storage on drop*, which makes the release panic-safe — a worker that panics
/// mid-parse still returns the buffer/slot:
/// - [`ChunkInner::Owned`] recycles its `Vec` back to the producer (when it is
///   the sole `Arc` owner) so the allocation's pages stay faulted-in, exactly as
///   the old `Arc::try_unwrap` recycle did.
/// - [`ChunkInner::Shared`] returns its shared-memory slot id to the
///   decompressor so the slot can be refilled.
pub struct Chunk {
    inner: ChunkInner,
}

enum ChunkInner {
    /// Heap buffer owned through an `Arc` (the in-process path). Recycled on drop
    /// when no other `Arc` clone is live.
    Owned {
        data: Option<Arc<Vec<u8>>>,
        recycle: Option<Sender<Vec<u8>>>,
    },
    /// A borrow of `len` bytes at `ptr` inside a shared-memory region owned by
    /// the parser, occupying slot `slot`. The slot id is returned on `release`
    /// when the chunk drops.
    Shared {
        ptr: *const u8,
        len: usize,
        slot: u32,
        release: Sender<u32>,
    },
}

// SAFETY: the `Shared` raw pointer addresses a shared-memory region that the
// parser keeps mapped for longer than any `Chunk` it hands out, and each slot
// has exactly one owner at a time (ownership is transferred through the
// descriptor/return pipes). Sending the `Chunk` across threads therefore never
// creates aliasing writes.
unsafe impl Send for Chunk {}

impl Chunk {
    /// An owned heap chunk. `recycle`, when `Some`, receives the inner `Vec` on
    /// drop (if this is the last `Arc` reference) for reuse by the producer.
    #[must_use]
    pub fn owned(data: Arc<Vec<u8>>, recycle: Option<Sender<Vec<u8>>>) -> Self {
        Chunk {
            inner: ChunkInner::Owned {
                data: Some(data),
                recycle,
            },
        }
    }

    /// A chunk borrowing `len` bytes at `ptr` from a shared-memory `slot`,
    /// returning the slot id via `release` on drop.
    ///
    /// # Safety
    /// `ptr..ptr+len` must point into a live shared region that outlives every
    /// `Chunk` produced from it, and nothing else may read or write `slot`'s
    /// bytes until this chunk is dropped (its slot returned).
    #[must_use]
    pub unsafe fn shared(ptr: *const u8, len: usize, slot: u32, release: Sender<u32>) -> Self {
        Chunk {
            inner: ChunkInner::Shared {
                ptr,
                len,
                slot,
                release,
            },
        }
    }

    #[inline]
    fn bytes(&self) -> &[u8] {
        match &self.inner {
            ChunkInner::Owned { data, .. } => data.as_ref().expect("present until drop"),
            // SAFETY: upheld by `Chunk::shared`'s contract — the region outlives
            // the chunk and the slot has a single owner, so this borrow is valid
            // and unaliased for the chunk's lifetime.
            ChunkInner::Shared { ptr, len, .. } => unsafe {
                std::slice::from_raw_parts(*ptr, *len)
            },
        }
    }
}

impl std::ops::Deref for Chunk {
    type Target = [u8];
    #[inline]
    fn deref(&self) -> &[u8] {
        self.bytes()
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        match &mut self.inner {
            ChunkInner::Owned { data, recycle } => {
                // Recycle the Vec only if we are its sole owner (a downstream CRC
                // validator may still hold an Arc clone); otherwise just drop it.
                if let (Some(arc), Some(tx)) = (data.take(), recycle.as_ref())
                    && let Ok(mut v) = Arc::try_unwrap(arc)
                {
                    v.clear();
                    let _ = tx.try_send(v);
                }
            }
            ChunkInner::Shared { slot, release, .. } => {
                let _ = release.try_send(*slot);
            }
        }
    }
}

/// One chunk's worth of FASTQ, split into per-role columns.
///
/// `names` is a [`StringPod`] of header lines with the leading `@` stripped via
/// an O(1) `cut_start`. `reads` is a [`DualStringPod`] fusing the sequence and
/// quality columns: because every record satisfies `seq.len() == qual.len()`,
/// the two share a single metadata column and the invariant is structural
/// rather than re-checked downstream. The separator (`+`) line is validated and
/// dropped.
///
/// A column is `Storage::FixedLength` when every entry in *this* emission shares
/// a length (the common fixed-read-length case) and `Variable` otherwise — the
/// pod builders start fixed on the first line's length and auto-promote on the
/// first mismatch.
///
/// Across the whole stream each column receives exactly one entry per record, in
/// order; every emitted chunk is record-aligned (`names.len() == reads.len()`).
#[derive(Debug)]
pub struct FastqChunk {
    pub names: StringPod,
    pub reads: DualStringPod,
}

/// A decode chunk handed to a demux worker, with the trailing partial line of
/// the previous chunk to stitch onto this chunk's leading partial line.
struct DemuxJob {
    idx: u64,
    chunk: Chunk,
    prev_tail: Vec<u8>,
}

/// A worker's phase-agnostic split of one chunk: four buckets holding the
/// chunk's lines indexed by `line_index_within_stream mod 4` (the worker does
/// not know which absolute role that is — the collector rotates), plus the
/// number of lines completed in the chunk (`= newline count`) so the collector
/// can accumulate the global phase.
struct DemuxResult {
    idx: u64,
    buckets: [StringPod; 4],
    lines: u64,
    /// At least one line in this chunk ended with `\r` (CRLF).
    saw_crlf: bool,
    /// At least one line in this chunk ended without `\r` (bare LF).
    saw_lf: bool,
}

/// Parse a channel of arbitrary already-decompressed byte chunks into columnar
/// FASTQ pods, emitting one [`FastqChunk`] per input chunk (record-aligned) to
/// `sink`. Blocks until `bytes_rx` is closed or the first error; closes `sink`
/// when it returns.
///
/// `demux_threads` sizes the parallel split pool (`0` → a small auto default).
/// Keep it small: a few workers are enough to keep the scan + copy off the
/// critical path, and more just oversubscribe and contend in the allocator.
///
/// Each [`Chunk`] releases its backing storage on drop — recycling an owned
/// `Vec` to its producer, or returning a shared-memory slot — so the caller
/// chooses the transport when it builds the chunks; this function just consumes
/// `&[u8]` and drops each chunk once its payload is copied into columns.
///
/// # Panics
/// when the fastq collector thread paniced?
pub fn parse_pods_from_channel(
    bytes_rx: Receiver<Chunk>,
    sink: Sender<FastqChunk>,
    demux_threads: usize,
) -> Result<()> {
    let demux_threads = if demux_threads == 0 {
        std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .unwrap_or(4)
            .min(4)
    } else {
        demux_threads
    }
    .max(1);

    // Demux pool — phase-agnostic per-chunk bucketing (the heavy scan + copy).
    let (job_tx, job_rx) = channel::bounded::<DemuxJob>(demux_threads * 2);
    let (done_tx, done_rx) = channel::bounded::<Result<DemuxResult>>(demux_threads * 2);
    let mut workers = Vec::with_capacity(demux_threads);
    for _ in 0..demux_threads {
        let job_rx = job_rx.clone();
        let done_tx = done_tx.clone();
        workers.push(std::thread::spawn(move || {
            for job in job_rx {
                let result = demux_chunk(job.idx, &job.chunk, &job.prev_tail);
                // Payload is copied into the columns now; drop the chunk to
                // release its storage (recycle the Vec / return the shm slot).
                drop(job.chunk);
                if done_tx.send(result).is_err() {
                    return;
                }
            }
        }));
    }
    drop(job_rx);
    drop(done_tx);

    // Collector — serial, in stream order: rotate buckets onto roles + emit.
    let (tail_tx, tail_rx) = channel::bounded::<Vec<u8>>(1);
    let collector = std::thread::spawn(move || collect(done_rx, &tail_rx, &sink));

    // Stage A — serial, cheap: thread the trailing-partial-line carry across
    // boundaries (one backward scan per chunk) and dispatch demux jobs.
    let mut carry: Vec<u8> = Vec::new();
    let mut idx: u64 = 0;
    for chunk in bytes_rx {
        match rposition_nl(&chunk) {
            Some(last_nl) => {
                let next_carry = chunk[last_nl + 1..].to_vec();
                let prev_tail = std::mem::take(&mut carry);
                if job_tx
                    .send(DemuxJob {
                        idx,
                        chunk,
                        prev_tail,
                    })
                    .is_err()
                {
                    break;
                }
                idx += 1;
                carry = next_carry;
            }
            None => {
                // No newline at all (pathological for FASTQ): the whole chunk is
                // the middle of one line. Fold it into the carry; emit nothing.
                carry.extend_from_slice(&chunk);
            }
        }
    }
    drop(job_tx);
    // Final unterminated line (file without a trailing newline), if any.
    let _ = tail_tx.send(carry);
    drop(tail_tx);

    for w in workers {
        let _ = w.join();
    }
    collector.join().expect("fastq collector thread panicked")
}

#[inline]
fn strip_cr(line: &[u8]) -> &[u8] {
    match line.last() {
        Some(b'\r') => &line[..line.len() - 1],
        _ => line,
    }
}

#[inline]
fn rposition_nl(data: &[u8]) -> Option<usize> {
    memchr::memrchr(b'\n', data)
}

/// FASTQ columns address their byte buffer with `u32` offsets, so any single
/// column — and therefore any single read's sequence or quality line — cannot
/// exceed `u32::MAX` bytes. Degenerate/adversarial input (a multi-GiB single
/// read, or a tiny chunk that inflates past 4 GiB) would otherwise overflow the
/// `u32` position in `StringPodBuilder::push` and panic; the guard below turns
/// that into a clean error instead.
const MAX_FASTQ_COLUMN_BYTES: usize = u32::MAX as usize;

/// Reject a push that would grow a FASTQ column past [`MAX_FASTQ_COLUMN_BYTES`]
/// with a clear error rather than letting the underlying `u32` cast panic.
#[inline]
fn fastq_len_guard(current: usize, add: usize) -> Result<()> {
    // `current` is always ≤ u32::MAX (every prior push was guarded), so this
    // sum cannot overflow usize on the 64-bit targets this crate supports.
    if current + add > MAX_FASTQ_COLUMN_BYTES {
        bail!("FASTQ read length exceeds the allowed maximum of 4 GiB");
    }
    Ok(())
}

#[inline]
fn push_line(bucket: &mut Option<StringPodBuilder>, est: usize, line: &[u8]) -> Result<()> {
    // Guard before `with_capacity`, which itself casts the entry length to u32
    // and would panic on a >4 GiB line before we ever reach `push`.
    let current = bucket.as_ref().map_or(0, StringPodBuilder::buffer_bytes);
    fastq_len_guard(current, line.len())?;
    bucket
        .get_or_insert_with(|| StringPodBuilder::with_capacity(line.len(), est))
        .push(line);
    Ok(())
}

/// Split one chunk into four buckets keyed by `line_index mod 4`, phase
/// unknown. The line that straddles the *previous* boundary is reassembled
/// from `prev_tail ++ this chunk's leading partial line` and pushed first into
/// bucket 3 — that is the line one position *before* the first fully-contained
/// line (local index 0 → bucket 0), so it shares bucket 0's predecessor slot.
/// After the collector's rotation this lands in the correct role column, in
/// order, with no further copy. `data` is guaranteed to contain ≥1 newline.
fn demux_chunk(idx: u64, data: &[u8], prev_tail: &[u8]) -> Result<DemuxResult> {
    let est = (data.len() / 300).max(16);
    let mut builders: [Option<StringPodBuilder>; 4] = [None, None, None, None];

    let first_nl = memchr::memchr(b'\n', data).expect("≥1 newline");

    // Track, per chunk, whether lines end with `\r` (CRLF) or bare LF, so the
    // collector can reject a file that mixes the two. `last()` is O(1), so this
    // is free for the common single-style file.
    let mut saw_crlf = false;
    let mut saw_lf = false;
    let mut observe = |line: &[u8], fallback: &[u8]| {
        // A line that ends right at the chunk start (empty `line`) inherits its
        // last byte from the stitched-on `fallback` (the previous chunk's tail).
        let ends_cr = match line.last() {
            Some(&b'\r') => true,
            Some(_) => false,
            None => fallback.last() == Some(&b'\r'),
        };
        if ends_cr {
            saw_crlf = true;
        } else {
            saw_lf = true;
        }
    };

    // Reassemble the boundary-straddling line and push it into bucket 3. Guard
    // its assembled length up front so an adversarial multi-GiB line is rejected
    // before we materialise the (equally multi-GiB) `split` buffer.
    let raw_head = &data[..first_nl];
    observe(raw_head, prev_tail);
    let head = strip_cr(raw_head);
    fastq_len_guard(0, prev_tail.len() + head.len())?;
    let mut split = Vec::with_capacity(prev_tail.len() + head.len());
    split.extend_from_slice(prev_tail);
    split.extend_from_slice(head);
    push_line(&mut builders[3], est, strip_cr(&split))?;

    // Fully-contained lines: between consecutive newlines, local index from 0.
    let mut lines: u64 = 1; // the boundary line, completed by `first_nl`
    // SIMD newline scan (memchr/AVX2) over the chunk body rather than a scalar
    // byte-by-byte `position`. This loop runs once per FASTQ line across the
    // entire decoded stream, so it dominates parse CPU — profiling the 22 GB
    // count-and-discard run showed ~120 s of user time in this scan+copy path.
    let body = &data[first_nl + 1..];
    let mut line_start = 0usize;
    for (local, rel) in memchr::memchr_iter(b'\n', body).enumerate() {
        let line = &body[line_start..rel];
        observe(line, b"");
        push_line(&mut builders[local & 3], est, strip_cr(line))?;
        lines += 1;
        line_start = rel + 1;
    }
    // body[line_start..] is this chunk's trailing partial line — carried by Stage A.

    // Reserve a little slack on every bucket so the collector's per-boundary
    // appends (≤3 lines completing a straddling record) land in place instead
    // of reallocating these (potentially multi-MB) buffers.
    let buckets = builders.map(|b| {
        let mut pod = b.map_or_else(StringPod::empty, StringPodBuilder::finish);
        pod.reserve_for_appends(4);
        pod
    });
    Ok(DemuxResult {
        idx,
        buckets,
        lines,
        saw_crlf,
        saw_lf,
    })
}

/// Role-indexed columns for one chunk: `[name, seq, plus, qual]` (role = line
/// index mod 4). The collector keeps the previous chunk's `Cols` as `held`
/// until the next chunk supplies the lines completing its trailing record.
type Cols = [StringPod; 4];

/// Validate and emit one chunk of *whole* records. `cols` must already hold
/// equal-length, record-aligned columns (the collector guarantees this). The
/// `+` separator is validated and dropped; the leading `@` is stripped O(1).
fn emit_records(cols: Cols, emit: &mut impl FnMut(FastqChunk)) -> Result<()> {
    let [mut names, seqs, plus, quals] = cols;
    debug_assert_eq!(names.len(), seqs.len());
    debug_assert_eq!(seqs.len(), quals.len());

    if names.is_empty() {
        return Ok(()); // nothing to emit
    }
    for name in &names {
        if name.first() != Some(&b'@') {
            bail!("header line does not start with '@'");
        }
        // A bare '@' line (no characters after the marker) is an empty read name.
        // Cheap to check here at parse time, before the O(1) '@' strip below.
        if name.len() < 2 {
            bail!("Empty name in input FASTQ. Verify your input files are proper FASTQ.");
        }
    }
    if !plus.is_empty() && !plus.iter().all(|p| p.first() == Some(&b'+')) {
        bail!("separator line does not start with '+'");
    }
    // Sequence and quality share the FASTQ per-entry length invariant, so fuse
    // them into a single DualStringPod (zero-copy — both byte buffers move in
    // as-is). `try_from_columns` verifies seq.len() == qual.len() for every
    // record and that the two layouts are a constant translation, surfacing any
    // mismatch as an error rather than emitting a malformed chunk.
    let reads = DualStringPod::try_from_columns(seqs, quals).map_err(|e| {
        anyhow!(
            "Sequence and quality must have the same length. Check your input fastq. \
             Wrapped FASTQ is not supported. Original error: {e:?}"
        )
    })?;
    names.cut_start(1, None); // drop the leading '@' from every header, O(1)
    emit(FastqChunk { names, reads });
    Ok(())
}

/// Absorb one in-order chunk. The demux buckets are rotated onto roles using
/// the running phase (`global_lines % 4`), then merged into `held` via the
/// "complete the previous chunk's straddling record from this chunk's head"
/// scheme: any record that begins in `held` but spills into this chunk is
/// finished by appending this chunk's leading lines onto `held` (≤3 tiny line
/// copies), and those same lines are front-skipped off this chunk. `held` then
/// holds only whole records and is emitted; this chunk (record-aligned at its
/// head) becomes the new `held`.
#[expect(clippy::cast_possible_wrap, reason = "can only be 0..4")]
#[expect(
    clippy::cast_possible_truncation,
    reason = "No support for 32bit machines"
)]
fn absorb(
    res: DemuxResult,
    held: &mut Option<Cols>,
    global_lines: &mut u64,
    emit: &mut impl FnMut(FastqChunk),
) -> Result<()> {
    let phase = (*global_lines % 4) as usize; // = held's trailing-record line count
    let g = phase as i64;
    // role r is held by bucket b where (g + 1 + b) % 4 == r.
    let src = |r: i64| ((r - g - 1).rem_euclid(4)) as usize;
    let mut b: [Option<StringPod>; 4] = res.buckets.map(Some);
    let mut cur: Cols = [
        b[src(0)].take().expect("rotation is a permutation"),
        b[src(1)].take().expect("rotation is a permutation"),
        b[src(2)].take().expect("rotation is a permutation"),
        b[src(3)].take().expect("rotation is a permutation"),
    ];
    let avail = res.lines as usize;
    *global_lines += res.lines;

    match held.take() {
        None => {
            // Only reachable at a record boundary (phase 0): this chunk starts a
            // fresh record run.
            debug_assert_eq!(phase, 0);
            *held = Some(cur);
        }
        Some(mut h) => {
            let need = (4 - phase) % 4; // lines to finish held's trailing record
            let take = need.min(avail);
            for role in phase..phase + take {
                // role ∈ [phase, 4): the straddling record's missing lines, in
                // order, at the head of `cur`. Move them onto `held`.
                let line = cur[role].get(0).to_vec();
                fastq_len_guard(h[role].buffer_bytes(), line.len())?;
                h[role].push(&line);
                cur[role].pop_front(1);
            }
            if take == need {
                // held's trailing record is complete → flush it.
                emit_records(h, emit)?;
                // `cur` (front-skipped past the lines we consumed) now starts at
                // a record boundary; it becomes the next `held`.
                *held = if avail - take > 0 { Some(cur) } else { None };
            } else {
                // Consumed all of `cur` without completing the record (tiny
                // chunk straddling >2 chunks); keep accumulating into `held`.
                *held = Some(h);
            }
        }
    }
    Ok(())
}

/// Flush at end of stream: fold the file's final unterminated line (if any,
/// from `tail_rx` / the leftover carry) onto `held`, then emit the remaining
/// whole records. Returns an error if the stream ends mid-record.
fn finish_eof(
    held: Option<Cols>,
    carry: &[u8],
    global_lines: u64,
    emit: &mut impl FnMut(FastqChunk),
) -> Result<()> {
    let Some(mut h) = held else {
        if !carry.is_empty() {
            // A non-empty carry with no completed lines at all means the stream
            // never contained a single newline — almost certainly not FASTQ.
            if global_lines == 0 {
                bail!(
                    "Parsing error: read all of file, but found no newlines. Check your input FASTQ file."
                );
            }
            bail!(
                "truncated FASTQ: incomplete record at end of stream (read {} complete reads before truncation)",
                global_lines / 4
            );
        }
        return Ok(());
    };
    if !carry.is_empty() {
        let role = (global_lines % 4) as usize;
        let line = strip_cr(carry);
        fastq_len_guard(h[role].buffer_bytes(), line.len())?;
        h[role].push(line);
    }
    let complete = h[0].len().min(h[1].len()).min(h[3].len());
    // Tolerate a single trailing blank line: a file that ends with an extra
    // newline (`…qual\n\n`) leaves one extra, *empty* name line beyond the last
    // whole record. The legacy parser ignored exactly this, so drop it rather
    // than reporting a truncated record.
    if h[0].len() == complete + 1
        && h[1].len() == complete
        && h[3].len() == complete
        && h[0].get(complete).is_empty()
    {
        for col in &mut h {
            col.truncate(complete);
        }
    }
    if h[0].len() != complete || h[1].len() != complete || h[3].len() != complete {
        bail!(
            "truncated FASTQ: incomplete record at end of stream (read {} complete reads before truncation)",
            global_lines / 4
        );
    }
    emit_records(h, emit)
}

/// Reorder demux results by chunk index and run the serial record-alignment
/// (`absorb` / `finish_eof`), emitting a record-aligned [`FastqChunk`] per
/// input chunk. Runs serially in stream order.
fn collect(
    done_rx: Receiver<Result<DemuxResult>>,
    tail_rx: &Receiver<Vec<u8>>,
    sink: &Sender<FastqChunk>,
) -> Result<()> {
    use std::collections::BTreeMap;
    let mut reorder: BTreeMap<u64, DemuxResult> = BTreeMap::new();
    let mut next: u64 = 0;
    let mut global_lines: u64 = 0;
    let mut held: Option<Cols> = None;
    // A well-formed FASTQ uses a single newline style throughout. Track whether
    // we've seen CRLF and/or bare-LF line endings and reject a file that mixes
    // them (e.g. a half-converted Windows file).
    let mut saw_crlf = false;
    let mut saw_lf = false;
    let mut emit = |chunk: FastqChunk| {
        let _ = sink.send(chunk);
    };

    // Fold one chunk's line-ending observations into the running state and
    // reject the first chunk that tips the file into mixed endings.
    let check_endings = |res: &DemuxResult, saw_crlf: &mut bool, saw_lf: &mut bool| -> Result<()> {
        *saw_crlf |= res.saw_crlf;
        *saw_lf |= res.saw_lf;
        if *saw_crlf && *saw_lf {
            bail!(
                "FASTQ uses inconsistent line endings (a mix of LF and CR+LF). \
                 Convert the file to a single newline style."
            );
        }
        Ok(())
    };

    for res in done_rx {
        let res = res?; // a worker hit invalid/oversized input — surface it
        reorder.insert(res.idx, res);
        while let Some(res) = reorder.remove(&next) {
            check_endings(&res, &mut saw_crlf, &mut saw_lf)?;
            absorb(res, &mut held, &mut global_lines, &mut emit)?;
            next += 1;
        }
    }
    // Any stragglers (shouldn't happen once done_rx is closed, but be safe).
    while let Some(res) = reorder.remove(&next) {
        check_endings(&res, &mut saw_crlf, &mut saw_lf)?;
        absorb(res, &mut held, &mut global_lines, &mut emit)?;
        next += 1;
    }

    let carry = tail_rx.recv().unwrap_or_default();
    // The final unterminated line counts toward the newline-style check too.
    if !carry.is_empty() {
        if carry.last() == Some(&b'\r') {
            saw_crlf = true;
        } else {
            saw_lf = true;
        }
        if saw_crlf && saw_lf {
            bail!(
                "FASTQ uses inconsistent line endings (a mix of LF and CR+LF). \
                 Convert the file to a single newline style."
            );
        }
    }
    finish_eof(held, &carry, global_lines, &mut emit)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Concatenated `(names, seqs, quals)` FASTQ columns, each entry
    /// newline-terminated.
    type FastqColumns = (Vec<u8>, Vec<u8>, Vec<u8>);

    /// Run the full demux + record-alignment over an explicit sequence of decode
    /// chunks (no threads) and return the concatenated `(names, seqs, quals)`
    /// columns, each entry newline-terminated. Lets the tests assert
    /// chunk-boundary independence and the complete-records invariant directly
    /// against the real `absorb` / `finish_eof` logic.
    fn split(chunks: &[&[u8]]) -> Result<FastqColumns> {
        let (mut names, mut seqs, mut quals) = (Vec::new(), Vec::new(), Vec::new());
        let mut emit = |c: FastqChunk| {
            for x in &c.names {
                names.extend_from_slice(x);
                names.push(b'\n');
            }
            for x in c.reads.iter_seq() {
                seqs.extend_from_slice(x);
                seqs.push(b'\n');
            }
            for x in c.reads.iter_qual() {
                quals.extend_from_slice(x);
                quals.push(b'\n');
            }
        };

        let mut held: Option<Cols> = None;
        let mut global_lines: u64 = 0;
        let mut carry: Vec<u8> = Vec::new();
        let mut idx: u64 = 0;
        for chunk in chunks {
            match rposition_nl(chunk) {
                Some(last_nl) => {
                    let next_carry = chunk[last_nl + 1..].to_vec();
                    let prev_tail = std::mem::take(&mut carry);
                    let res = demux_chunk(idx, chunk, &prev_tail)?;
                    absorb(res, &mut held, &mut global_lines, &mut emit)?;
                    idx += 1;
                    carry = next_carry;
                }
                None => carry.extend_from_slice(chunk),
            }
        }
        finish_eof(held, &carry, global_lines, &mut emit)?;
        Ok((names, seqs, quals))
    }

    /// A small multi-record FASTQ stream with variable read lengths.
    fn sample() -> Vec<u8> {
        let mut v = Vec::new();
        for i in 0..50u32 {
            let len = 1 + (i as usize % 11);
            let seq = "A".repeat(len);
            let qual = "I".repeat(len);
            v.extend_from_slice(format!("@read.{i} desc\n{seq}\n+\n{qual}\n").as_bytes());
        }
        v
    }

    fn expected_columns(data: &[u8]) -> FastqColumns {
        let (mut n, mut s, mut q) = (Vec::new(), Vec::new(), Vec::new());
        let lines: Vec<&[u8]> = data.split(|&b| b == b'\n').collect();
        // last element is empty (trailing newline) — records are groups of 4.
        for rec in lines.chunks(4) {
            if rec.len() < 4 {
                break;
            }
            n.extend_from_slice(&rec[0][1..]); // drop '@'
            n.push(b'\n');
            s.extend_from_slice(rec[1]);
            s.push(b'\n');
            q.extend_from_slice(rec[3]);
            q.push(b'\n');
        }
        (n, s, q)
    }

    #[test]
    fn whole_stream_splits_into_expected_columns() {
        let data = sample();
        let got = split(&[&data]).expect("valid fastq");
        assert_eq!(got, expected_columns(&data));
    }

    #[test]
    fn columns_are_independent_of_chunk_boundaries() {
        let data = sample();
        let whole = split(&[&data]).expect("valid fastq");

        // Irregular, data-driven chunking.
        let mut irregular: Vec<&[u8]> = Vec::new();
        let mut i = 0;
        while i < data.len() {
            let step = 1 + (data[i] as usize % 17);
            let end = (i + step).min(data.len());
            irregular.push(&data[i..end]);
            i = end;
        }
        assert_eq!(split(&irregular).expect("irregular"), whole);

        // Byte-by-byte: forces the partial-record carry-stitch at every position.
        let bb: Vec<&[u8]> = data.chunks(1).collect();
        assert_eq!(split(&bb).expect("byte-by-byte"), whole);
    }

    #[test]
    fn end_to_end_through_the_channel() {
        let data = sample();
        let (tx, rx) = channel::bounded::<Chunk>(8);
        // Feed the payload as several awkwardly-sized chunks.
        let producer = std::thread::spawn(move || {
            for piece in data.chunks(7) {
                if tx
                    .send(Chunk::owned(Arc::new(piece.to_vec()), None))
                    .is_err()
                {
                    break;
                }
            }
        });

        let (chunk_tx, chunk_rx) = channel::bounded::<FastqChunk>(8);
        let parser = std::thread::spawn(move || parse_pods_from_channel(rx, chunk_tx, 2));

        let (mut n, mut s, mut q) = (Vec::new(), Vec::new(), Vec::new());
        for c in chunk_rx {
            assert_eq!(c.names.len(), c.reads.len());
            for x in &c.names {
                n.extend_from_slice(x);
                n.push(b'\n');
            }
            for x in c.reads.iter_seq() {
                s.extend_from_slice(x);
                s.push(b'\n');
            }
            for x in c.reads.iter_qual() {
                q.extend_from_slice(x);
                q.push(b'\n');
            }
        }
        producer.join().expect("producer panicked");
        parser.join().expect("parser panicked").expect("parse ok");

        assert_eq!((n, s, q), expected_columns(&sample()));
    }

    #[test]
    fn truncated_record_is_rejected() {
        // Three lines: a record missing its quality line.
        let data = b"@r\nACGT\n+\n";
        let err = split(&[data]).expect_err("should be truncated");
        assert!(err.to_string().contains("truncated"), "got: {err}");
    }

    #[test]
    fn missing_at_prefix_is_rejected() {
        let data = b"r\nACGT\n+\nIIII\n";
        let err = split(&[data]).expect_err("header must start with '@'");
        assert!(err.to_string().contains("'@'"), "got: {err}");
    }

    #[test]
    fn len_guard_accepts_up_to_the_limit_and_rejects_past_it() {
        fastq_len_guard(MAX_FASTQ_COLUMN_BYTES - 10, 10).unwrap();
        fastq_len_guard(0, MAX_FASTQ_COLUMN_BYTES).unwrap();
        let err = fastq_len_guard(0, MAX_FASTQ_COLUMN_BYTES + 1).unwrap_err();
        assert!(err.to_string().contains("4 GiB"), "got: {err}");
        assert!(fastq_len_guard(MAX_FASTQ_COLUMN_BYTES, 1).is_err());
    }

    // ── Adversarial / degenerate input ──────────────────────────────────────
    //
    // Ported from rusty-rapidgzip's `tests/adversarial.rs`. These probe the u32
    // per-column byte limit of the StringPod columnar layout (positions are u32,
    // so any single column — hence any single read line — is capped at 4 GiB)
    // and large-input correctness across arbitrary chunk boundaries.

    /// A large run of fixed-length reads, fed through the channel in many small
    /// chunks, must decode correctly — every read recovered, in order, with the
    /// right count — across both single- and multi-threaded demux. The
    /// fixed-length fast path (`Storage::FixedLength`) is exercised throughout.
    ///
    /// In the original this also probed the gzip pipeline's 64 MiB speculative
    /// reservation; that is a decoder detail with no meaning for this
    /// source-agnostic parser, so here it simply guards large-input correctness
    /// across arbitrary chunk boundaries.
    #[test]
    fn large_fixed_length_input_decodes_correctly() {
        const READS: usize = 50_000;
        const LEN: usize = 256;
        const BASE: u8 = b'A';

        // Each record: `@` + 256 'A' (name) / 256 'A' (seq) / `+` / 256 'A' (qual).
        let mut payload = Vec::new();
        for _ in 0..READS {
            payload.push(b'@');
            payload.resize(payload.len() + LEN, BASE);
            payload.push(b'\n');
            payload.resize(payload.len() + LEN, BASE);
            payload.extend_from_slice(b"\n+\n");
            payload.resize(payload.len() + LEN, BASE);
            payload.push(b'\n');
        }
        let payload = Arc::new(payload);

        for threads in [1usize, 4] {
            let (tx, rx) = channel::bounded::<Chunk>(8);
            let p = Arc::clone(&payload);
            let producer = std::thread::spawn(move || {
                for piece in p.chunks(64 * 1024) {
                    if tx
                        .send(Chunk::owned(Arc::new(piece.to_vec()), None))
                        .is_err()
                    {
                        break;
                    }
                }
            });
            let (chunk_tx, chunk_rx) = channel::bounded::<FastqChunk>(8);
            let parser = std::thread::spawn(move || parse_pods_from_channel(rx, chunk_tx, threads));

            // seq/qual share one metadata column in the DualStringPod, so their
            // counts are structurally equal; assert names line up too.
            let all_base = |b: &[u8]| b.len() == LEN && b.iter().all(|&c| c == BASE);
            let mut reads = 0usize;
            for c in chunk_rx {
                assert_eq!(c.names.len(), c.reads.len(), "names vs reads (t={threads})");
                for name in &c.names {
                    assert!(all_base(name), "name (t={threads})");
                }
                for seq in c.reads.iter_seq() {
                    assert!(all_base(seq), "seq (t={threads})");
                }
                for qual in c.reads.iter_qual() {
                    assert!(all_base(qual), "qual (t={threads})");
                }
                reads += c.reads.len();
            }
            producer.join().expect("producer panicked");
            parser
                .join()
                .expect("parser panicked")
                .expect("decode failed");
            assert_eq!(reads, READS, "read count (t={threads})");
        }
    }

    /// A single read whose sequence line exceeds the u32 per-column byte limit
    /// must be rejected with a clean error, not a panic in the `StringPod` `u32`
    /// position cast (the original bug this guards).
    ///
    /// Heavy: the parser accumulates >4 GiB of carry before the offending line is
    /// assembled, so it needs several GiB of RAM. Ignored by default; run with:
    ///
    /// ```text
    /// cargo test -p fastqrab-io --release -- --ignored read_exceeding_u32
    /// ```
    #[test]
    #[ignore = "accumulates >4 GiB and needs several GiB of RAM; run with --ignored"]
    fn read_exceeding_u32_fails_gracefully() {
        let (tx, rx) = channel::bounded::<Chunk>(8);
        let producer = std::thread::spawn(move || {
            // `@r\n` then a single sequence line longer than u32::MAX, then its
            // terminating newline: the >4 GiB line is the boundary-straddling
            // line whose assembly trips the guard.
            if tx
                .send(Chunk::owned(Arc::new(b"@r\n".to_vec()), None))
                .is_err()
            {
                return;
            }
            let block = vec![b'A'; 64 * 1024 * 1024];
            let target: u64 = u64::from(u32::MAX) + (1 << 20);
            let mut sent: u64 = 0;
            while sent < target {
                if tx
                    .send(Chunk::owned(Arc::new(block.clone()), None))
                    .is_err()
                {
                    return;
                }
                sent += block.len() as u64;
            }
            let _ = tx.send(Chunk::owned(Arc::new(b"\n+\n".to_vec()), None));
        });

        let (chunk_tx, chunk_rx) = channel::bounded::<FastqChunk>(8);
        let parser = std::thread::spawn(move || parse_pods_from_channel(rx, chunk_tx, 2));

        // No whole record ever completes, so just drain until the sink closes.
        for _ in chunk_rx {}
        producer.join().expect("producer panicked");
        let err = parser
            .join()
            .expect("parser panicked")
            .expect_err("expected a graceful error, not success");
        assert!(
            err.to_string()
                .contains("FASTQ read length exceeds the allowed maximum of 4 GiB"),
            "unexpected error: {err}"
        );
    }
}
