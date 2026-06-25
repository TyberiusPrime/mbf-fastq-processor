//! fastqrab's out-of-process gzip/zstd decompressor.
//!
//! Parses args, decodes the input (gzip via `rusty_rapidgzip::read_gz`, zstd via
//! [`read_zstd`]) into a channel of decompressed byte chunks, and ships that
//! channel out — either as raw bytes on stdout (pipe mode) or memcpy'd into a
//! shared-memory ring (shm mode). fastqrab spawns this as a sibling binary (see
//! fastqrab-io `spawn_decompressor` / `find_decompressor`) so the decoders'
//! `unsafe` (rapidgzip, libzstd) runs in an isolated process. The transport
//! (`run_pipe` / `run_shm`) is format-agnostic — it only consumes the decoded
//! chunk channel — so adding zstd was purely a second producer. Kept to a small
//! CLI shape (positional input, `--format`, `-P`, `--chunk-size`, `-v`).

use std::io::{Read as _, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use crossbeam::channel::{Receiver, Sender, bounded};
use rusty_rapidgzip::{Config, Verbosity, elapsed_since_start, read_gz};

/// Input compression format. The transport downstream is identical for both; the
/// only difference is which producer fills the decoded-chunk channel.
#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
enum Format {
    Gzip,
    Zstd,
}

#[derive(Parser, Debug)]
#[command(name = "fastqrab-decompressor", version)]
struct Args {
    /// Input file, or `-` for stdin.
    input: PathBuf,
    /// Input compression format. `gzip` uses the parallel rapidgzip decoder;
    /// `zstd` uses a single-threaded libzstd streaming decode (so `-P` is
    /// ignored). The parent already sniffs the format and passes it explicitly.
    #[arg(long, value_enum, default_value_t = Format::Gzip)]
    format: Format,
    /// Number of worker threads (0 = auto). Gzip only; zstd decode is serial.
    #[arg(short = 'P', long, default_value_t = 0)]
    threads: usize,
    /// Approximate chunk size in bytes.
    #[arg(long, default_value_t = 4 * 1024 * 1024)]
    chunk_size: usize,
    /// Print per-member / per-chunk diagnostics to stderr.
    #[arg(short = 'v', long)]
    verbose: bool,

    // ── Shared-memory output (Unix only) ─────────────────────────────────────
    //
    // When `--shm-fd` is given, finished chunks are memcpy'd into slots of an
    // inherited shared-memory region (instead of `write_all`ing the bytes to
    // stdout), and a tiny `(slot, len)` descriptor is written to stdout per slot.
    // Freed slot ids flow back on stdin. Without `--shm-fd` the tool behaves
    // exactly as before (raw decompressed bytes on stdout), so the standalone
    // gunzip-equivalence path is untouched.
    /// File descriptor of the inherited shared-memory region (`memfd`).
    #[arg(long)]
    shm_fd: Option<i32>,
    /// Number of slots in the shared-memory region.
    #[arg(long)]
    shm_slots: Option<usize>,
    /// Size of each shared-memory slot in bytes.
    #[arg(long)]
    shm_slot_size: Option<usize>,
}

/// Emit a one-line throughput report to stderr. Split out so the lossy
/// byte-count → MB/s casts live behind a single documented `expect`.
#[expect(
    clippy::cast_precision_loss,
    reason = "MB/s is a human-facing diagnostic; f64 precision loss on byte counts is irrelevant"
)]
fn report_throughput(bytes_since_last: u64, total_bytes: u64, elapsed_secs: f64, since_start: f64) {
    let mbps = bytes_since_last as f64 / elapsed_secs / (1024.0 * 1024.0);
    let avg = total_bytes as f64 / since_start / (1024.0 * 1024.0);
    eprintln!(
        "[decompressor +{:.2}s] {:.1} MB/s (avg {:.1} MB/s, {:.1} MB total)",
        elapsed_since_start(),
        mbps,
        avg,
        total_bytes as f64 / (1024.0 * 1024.0),
    );
}

fn main() -> Result<()> {
    let args = Args::parse();
    // Recycle channel: drained output Vecs flow back to workers so pages
    // stay faulted. Capacity ~= worker pool size; if it fills, drop the
    // Vec rather than block stdout.
    let recycle_cap = std::thread::available_parallelism().map_or(4, std::num::NonZero::get) * 2;
    let (recycle_tx, recycle_rx) = bounded::<Vec<u8>>(recycle_cap);

    let (tx, rx) = bounded::<Arc<Vec<u8>>>(16);

    // `-` means stdin. `/dev/stdin` lets the decoder open it as a normal path:
    // when stdin is a pipe it routes to the streaming decoder, and when it's a
    // redirected regular file (`< foo.gz`) it still mmaps.
    let input = if args.input.as_os_str() == "-" {
        PathBuf::from("/dev/stdin")
    } else {
        args.input.clone()
    };

    // Producer: fill `tx` with decoded chunks, recycling drained buffers from
    // `recycle_rx` to keep their pages faulted. Both formats feed the identical
    // `run_shm` / `run_pipe` transport below; only the decode backend differs.
    let chunk_size = args.chunk_size;
    let verbose = args.verbose;
    let producer = match args.format {
        Format::Gzip => {
            let cfg = Config {
                num_threads: args.threads,
                chunk_size_bytes: chunk_size,
                verbose: if verbose {
                    Verbosity::On
                } else {
                    Verbosity::Off
                },
                recycle_rx: Some(recycle_rx),
                recycle_tx: Some(recycle_tx.clone()),
            };
            std::thread::spawn(move || read_gz(&input, tx, cfg).map(|_| ()).map_err(Into::into))
        }
        Format::Zstd => std::thread::spawn(move || read_zstd(&input, &tx, &recycle_rx, chunk_size)),
    };

    match args.shm_fd {
        Some(fd) => {
            let slots = args
                .shm_slots
                .context("--shm-slots is required together with --shm-fd")?;
            let slot_size = args
                .shm_slot_size
                .context("--shm-slot-size is required together with --shm-fd")?;
            run_shm(fd, slots, slot_size, &rx, &recycle_tx)?;
        }
        None => run_pipe(&rx, &recycle_tx, args.verbose)?,
    }
    // Close recycle_tx so any worker still blocked on recv exits cleanly
    // (workers use try_recv though, so this is belt-and-braces).
    drop(recycle_tx);

    producer.join().expect("producer thread panicked")?;
    Ok(())
}

/// Streaming zstd producer: decode `input` and feed `tx` decoded byte chunks of
/// up to `chunk_size`, mirroring `read_gz`'s channel contract (stream order; the
/// channel closes when `tx` drops on return) and its buffer recycling (drained
/// `Vec`s return on `recycle_rx`, so their pages stay faulted). libzstd decode is
/// single-threaded, so unlike the gzip path there is no worker pool — just this
/// one decode loop feeding the shared `run_shm` / `run_pipe` transport.
fn read_zstd(
    input: &std::path::Path,
    tx: &Sender<Arc<Vec<u8>>>,
    recycle_rx: &Receiver<Vec<u8>>,
    chunk_size: usize,
) -> Result<()> {
    let file = std::fs::File::open(input)
        .with_context(|| format!("opening {} for zstd decode", input.display()))?;
    // `Decoder` reads through concatenated frames by default (like `zstd -d`).
    let mut decoder =
        zstd::stream::read::Decoder::new(file).context("initializing zstd decoder")?;
    loop {
        // Reuse a recycled buffer when available so its pages stay faulted; only
        // the regrown tail of a previously-truncated buffer is re-zeroed.
        let mut buf = recycle_rx.try_recv().unwrap_or_default();
        buf.clear();
        buf.resize(chunk_size, 0);
        let mut filled = 0usize;
        while filled < chunk_size {
            let n = decoder.read(&mut buf[filled..])?;
            if n == 0 {
                break;
            }
            filled += n;
        }
        if filled == 0 {
            break; // clean EOF, nothing pending
        }
        buf.truncate(filled);
        if tx.send(Arc::new(buf)).is_err() {
            break; // consumer hung up
        }
        if filled < chunk_size {
            break; // short fill ⇒ EOF
        }
    }
    Ok(())
}

/// Original pipe output: stream each finished chunk's bytes to stdout. Used
/// whenever `--shm-fd` is absent, so the standalone tool stays byte-identical to
/// `gunzip` output.
fn run_pipe(
    rx: &Receiver<Arc<Vec<u8>>>,
    recycle_tx: &Sender<Vec<u8>>,
    verbose: bool,
) -> Result<()> {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let start = std::time::Instant::now();
    let mut last_report = start;
    let mut bytes_since_last: u64 = 0;
    let mut total_bytes: u64 = 0;
    for chunk in rx {
        out.write_all(&chunk)?;
        bytes_since_last += chunk.len() as u64;
        total_bytes += chunk.len() as u64;
        if verbose {
            let now = std::time::Instant::now();
            let elapsed = now.duration_since(last_report);
            if elapsed.as_secs_f64() >= 1.0 {
                report_throughput(
                    bytes_since_last,
                    total_bytes,
                    elapsed.as_secs_f64(),
                    now.duration_since(start).as_secs_f64(),
                );
                last_report = now;
                bytes_since_last = 0;
            }
        }
        // Recycle the inner Vec only if no other Arc reference is live (the
        // CRC validator may still hold one). When it doesn't, the Vec just
        // gets dropped here and the worker allocates fresh next iteration.
        if let Ok(v) = Arc::try_unwrap(chunk) {
            let _ = recycle_tx.try_send(v);
        }
    }
    Ok(())
}

/// Shared-memory output: memcpy each finished chunk into free slots of the
/// inherited `memfd` region and emit one `(slot, len)` descriptor per slot on
/// stdout; freed slot ids return on stdin. Replaces the two bulk pipe copies
/// (kernel-in + consumer-out) with a single plain memcpy and lets the consumer
/// parse in place.
///
/// Slot lifecycle: free → we own (memcpy in) → descriptor sent → consumer owns
/// (parse out) → slot id returned on stdin → free. Exactly one owner at a time.
#[cfg(unix)]
fn run_shm(
    fd: i32,
    slots: usize,
    slot_size: usize,
    rx: &Receiver<Arc<Vec<u8>>>,
    recycle_tx: &Sender<Vec<u8>>,
) -> Result<()> {
    use std::io::Read as _;

    let total = slots
        .checked_mul(slot_size)
        .context("shared-memory region size overflow")?;
    // SAFETY: `fd` is the memfd the parent created and we inherited; mapping it
    // `MAP_SHARED` makes our writes visible to the parent. The mapping lives for
    // the rest of the process, longer than any descriptor we emit for it.
    let base = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            total,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if base == libc::MAP_FAILED {
        bail!(
            "mmap of shared-memory region failed: {}",
            std::io::Error::last_os_error()
        );
    }
    let base = base.cast::<u8>();

    // Free-slot pool: all slots start free. The stdin reader thread returns
    // consumer-released slot ids here; the main loop blocks on `recv` for free
    // backpressure (no spinning) once every slot is in flight.
    let (free_tx, free_rx) = crossbeam::channel::unbounded::<u32>();
    for s in 0..slots {
        free_tx
            .send(u32::try_from(s).expect("slot count fits in u32"))
            .expect("receiver is live");
    }

    // stdin reader: the consumer writes freed slot ids (u32 LE) back to us; feed
    // them into the free pool. Exits on EOF (consumer gone) or a closed pool.
    let stdin_reader = std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        let mut buf = [0u8; 4];
        // Stops on the first read error (EOF when the consumer closes the return
        // pipe) or once the free pool is closed.
        while stdin.read_exact(&mut buf).is_ok() {
            if free_tx.send(u32::from_le_bytes(buf)).is_err() {
                break;
            }
        }
    });

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for chunk in rx {
        let mut off = 0usize;
        // A chunk larger than one slot is split across several slots, one
        // descriptor each (the consumer tolerates arbitrary chunk boundaries).
        while off < chunk.len() {
            let n = (chunk.len() - off).min(slot_size);
            let slot = free_rx.recv().expect("free pool stays open until EOF");
            // SAFETY: `slot < slots` (only valid ids ever enter the pool) and
            // `n <= slot_size`, so the destination range is inside the mapping.
            // We own this slot exclusively until its descriptor is sent below.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    chunk.as_ptr().add(off),
                    base.add(slot as usize * slot_size),
                    n,
                );
            }
            write_descriptor(
                &mut out,
                slot,
                u32::try_from(n).expect("n <= slot_size, a usize that fit a slot"),
            )?;
            off += n;
        }
        // Recycle the inner Vec when we're its sole owner, exactly as pipe mode.
        if let Ok(v) = Arc::try_unwrap(chunk) {
            let _ = recycle_tx.try_send(v);
        }
    }
    // EOF sentinel: slot = u32::MAX (never a real slot id), len = 0.
    write_descriptor(&mut out, u32::MAX, 0)?;
    out.flush()?;
    // Don't join the stdin reader: it may still be blocked reading slot returns
    // from a consumer that has already drained everything. Process exit reaps it.
    drop(stdin_reader);
    Ok(())
}

#[cfg(not(unix))]
fn run_shm(
    _fd: i32,
    _slots: usize,
    _slot_size: usize,
    _rx: &Receiver<Arc<Vec<u8>>>,
    _recycle_tx: &Sender<Vec<u8>>,
) -> Result<()> {
    bail!("shared-memory output is only supported on Unix");
}

/// Write one 8-byte little-endian `(slot, len)` descriptor and flush. The
/// control channel is one descriptor per multi-MiB slot, so the per-write flush
/// is negligible — and necessary: the consumer blocks reading exactly 8 bytes,
/// so an unflushed descriptor (stdout is block/line-buffered) would deadlock it
/// (it can't return a slot it never sees).
fn write_descriptor(out: &mut impl Write, slot: u32, len: u32) -> Result<()> {
    let mut desc = [0u8; 8];
    desc[0..4].copy_from_slice(&slot.to_le_bytes());
    desc[4..8].copy_from_slice(&len.to_le_bytes());
    out.write_all(&desc)?;
    out.flush()?;
    Ok(())
}
