//! Shared, format-agnostic shared-memory chunk reader.
//!
//! Both the FASTQ and FASTA pod parsers consume an identical stream of borrowed
//! [`Chunk`]s from the out-of-process decompressor's shared-memory ring; only the
//! per-format parsing of those bytes differs. This module owns the *transport*:
//! it spawns the decompressor in shm mode, stands up the descriptor-reader and
//! slot-return threads, and hands back a `Receiver<Chunk>` plus everything the
//! parser must keep alive (the mapped region) or join at EOF (the threads and the
//! child process).

use std::sync::Arc;
use std::thread::JoinHandle;

use anyhow::Result;
use crossbeam::channel::{self, Receiver};

use crate::io::input::{DecompressorFormat, ShmDecompressor, ShmRegion, spawn_decompressor_shm};
use crate::io::parsers::ThreadCount;
use crate::io::pod_parser::{Chunk, ChunkRegion};

/// Whether the shared-memory decompressor transport is enabled. On by default;
/// `FASTQRAB_DECOMP_SHM=0` forces the legacy pipe path (A/B and field escape
/// hatch).
pub(crate) fn shm_enabled() -> bool {
    !matches!(std::env::var("FASTQRAB_DECOMP_SHM").as_deref(), Ok("0"))
}

/// Shared-memory slot size in bytes (`FASTQRAB_DECOMP_SHM_SLOT_SIZE`, default
/// 8 MiB — comfortably above the decoder's ~4 MiB chunk so a chunk usually fits
/// one slot). Tunable; tests shrink it to force the multi-slot chunk-split path.
pub(crate) fn shm_slot_size() -> usize {
    std::env::var("FASTQRAB_DECOMP_SHM_SLOT_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n > 0)
        .unwrap_or(8 * 1024 * 1024)
}

/// Number of shared-memory slots (`FASTQRAB_DECOMP_SHM_SLOTS`, default `fallback`
/// computed from the thread counts). Tunable; tests shrink it to a tiny ring to
/// stress backpressure / recycling. The pipeline is deadlock-free for any ring
/// size ≥ 1 (consumers never wait on a slot to finish).
pub(crate) fn shm_slot_count(fallback: usize) -> usize {
    std::env::var("FASTQRAB_DECOMP_SHM_SLOTS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|&n| n >= 1)
        .unwrap_or(fallback)
}

/// The live shm transport: a channel of borrowed [`Chunk`]s plus everything that
/// must outlive the chunks (`region`) or be joined at EOF (the two threads and
/// the `child`). The owning parser drains `bytes_rx` through its format parser,
/// then joins the handles and waits on the child to surface a non-zero exit.
pub(crate) struct ShmChunkReader {
    /// Borrowed chunks, one per `(slot, len)` descriptor, in stream order.
    pub bytes_rx: Receiver<Chunk>,
    /// Reads descriptors from the child and wraps each slot as a borrowed chunk.
    pub reader_handle: JoinHandle<Result<()>>,
    /// Relays freed slot ids back to the child's stdin.
    pub slot_writer_handle: JoinHandle<()>,
    /// The decompressor child (waited on at EOF to surface a non-zero exit).
    pub child: std::process::Child,
    /// Keeps the shared mapping alive for as long as any borrowed chunk lives.
    pub region: Arc<ShmRegion>,
}

/// Spawn the decompressor in shm mode for `path` (decoding `format`) and stand up
/// the descriptor-reader + slot-return threads. `parallel_hint` is the consuming
/// parser's worker count, used (with `thread_count`) only to size the slot ring.
pub(crate) fn spawn_shm_chunk_reader(
    path: &std::path::Path,
    format: DecompressorFormat,
    thread_count: ThreadCount,
    parallel_hint: usize,
) -> Result<ShmChunkReader> {
    use std::io::{Read as _, Write as _};

    // Slots are sized a bit larger than the decoder's ~4 MiB chunk so the common
    // chunk lands in a single slot (an oversized chunk still splits). The region
    // is `memfd`-backed and sparse, so unused slots cost no physical memory —
    // only in-flight slots fault in — and we can size the ring generously.
    let slot_size = shm_slot_size();
    let depth = thread_count.0.get().max(parallel_hint);
    let slots = shm_slot_count((depth * 2 + 4).clamp(8, 64));

    let ShmDecompressor {
        region,
        mut descriptors,
        mut slot_return,
        child,
        slots,
        slot_size,
    } = spawn_decompressor_shm(path, format, thread_count, slots, slot_size)?;

    // `bytes_tx` capacity ≥ `slots` guarantees the descriptor reader can always
    // enqueue an in-flight chunk without blocking, so the consumers (which never
    // wait on a slot to finish) keep draining and returning slots — no deadlock
    // for any ring size ≥ 1.
    let (bytes_tx, bytes_rx) = channel::bounded::<Chunk>(slots);
    // Slot-return channel: a `Chunk`'s drop pushes its freed slot id here; the
    // writer thread relays them to the child's stdin.
    let (slot_ret_tx, slot_ret_rx) = channel::unbounded::<u32>();

    let region_for_reader = Arc::clone(&region);
    let reader_handle = std::thread::spawn(move || -> Result<()> {
        let mut desc = [0u8; 8];
        loop {
            if let Err(e) = descriptors.read_exact(&mut desc) {
                // EOF before the sentinel ⇒ the decompressor died; surface it
                // (its stderr is inherited, so the real cause is already shown).
                // cov:excl-start
                return Err(anyhow::Error::new(e)
                    .context("fastqrab-decompressor closed before sending its EOF sentinel"));
                // cov:excl-end
            }
            let slot = u32::from_le_bytes(desc[0..4].try_into().expect("4 bytes"));
            let len = u32::from_le_bytes(desc[4..8].try_into().expect("4 bytes")) as usize;
            if slot == u32::MAX {
                break; // EOF sentinel
            }
            // The chunk borrows `len` bytes at this slot's offset and holds an
            // `Arc` clone of the region, so the mapping outlives the chunk by
            // construction; `ShmRegion::slice` bounds-checks the range. The
            // decompressor only emits `slot < slots` and `len <= slot_size`, and
            // won't reuse the slot until we return its id (on chunk drop), so the
            // borrow is single-owner for the chunk's lifetime.
            let offset = slot as usize * slot_size;
            let region: Arc<dyn ChunkRegion> = region_for_reader.clone();
            let chunk = Chunk::shared(region, offset, len, slot, slot_ret_tx.clone());
            if bytes_tx.send(chunk).is_err() {
                break; // consumer hung up (a downstream error) // cov:excl-line
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

    Ok(ShmChunkReader {
        bytes_rx,
        reader_handle,
        slot_writer_handle,
        child,
        region,
    })
}
