//! fastqrab's gzip/zstd decompressor, run as a subprocess via `fastqrab __decompressor`.
//!
//! Parses args, decodes the input (gzip via `rusty_rapidgzip::read_gz`, zstd via
//! [`read_zstd`]) into a channel of decompressed byte chunks, and ships that
//! channel out — either as raw bytes on stdout (pipe mode) or memcpy'd into a
//! shared-memory ring (shm mode). fastqrab spawns itself with `__decompressor` so
//! the decoders' `unsafe` (rapidgzip, libzstd) runs in an isolated process.

use std::io::{Read as _, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use crossbeam::channel::{Receiver, Sender, bounded};
use rusty_rapidgzip::{Config, Verbosity, elapsed_since_start, read_gz};

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
    /// Input compression format.
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

/// Entry point when invoked as `fastqrab __decompressor <args...>`.
///
/// # Errors
/// Propagates decode, IO, or spawn errors.
///
/// # Panics
/// If the producer thread panics.
pub fn run() -> Result<()> {
    // Invoked as: fastqrab __decompressor <decompressor-args...>
    // Drop argv[1] ("__decompressor") so clap sees only the decompressor's args.
    let mut raw = std::env::args_os();
    let argv0 = raw.next().unwrap_or_default();
    let _ = raw.next(); // drop "__decompressor"
    let args = Args::parse_from(std::iter::once(argv0).chain(raw));

    let recycle_cap = std::thread::available_parallelism().map_or(4, std::num::NonZero::get) * 2;
    let (recycle_tx, recycle_rx) = bounded::<Vec<u8>>(recycle_cap);
    let (tx, rx) = bounded::<Arc<Vec<u8>>>(16);

    let input = if args.input.as_os_str() == "-" {
        PathBuf::from("/dev/stdin")
    } else {
        args.input.clone()
    };


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
            run_shm(fd, slots, slot_size, &rx, &recycle_tx, producer)?;
            drop(recycle_tx);
        }
        None => {
            run_pipe(&rx, &recycle_tx, args.verbose)?;
            drop(recycle_tx);
            producer.join().expect("producer thread panicked")?;
        }
    }
    Ok(())
}

fn read_zstd(
    input: &std::path::Path,
    tx: &Sender<Arc<Vec<u8>>>,
    recycle_rx: &Receiver<Vec<u8>>,
    chunk_size: usize,
) -> Result<()> {
    let file = std::fs::File::open(input)
        .with_context(|| format!("opening {} for zstd decode", input.display()))?;
    let mut decoder =
        zstd::stream::read::Decoder::new(file).context("initializing zstd decoder")?;
    loop {
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
            break;
        }
        buf.truncate(filled);
        if tx.send(Arc::new(buf)).is_err() {
            break;
        }
        if filled < chunk_size {
            break;
        }
    }
    Ok(())
}

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
        if let Err(e) = out.write_all(&chunk) {
            if is_consumer_gone(&e) {
                return Ok(());
            }
            return Err(e.into());
        }
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
        if let Ok(v) = Arc::try_unwrap(chunk) {
            let _ = recycle_tx.try_send(v);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn run_shm(
    fd: i32,
    slots: usize,
    slot_size: usize,
    rx: &Receiver<Arc<Vec<u8>>>,
    recycle_tx: &Sender<Vec<u8>>,
    producer: std::thread::JoinHandle<Result<()>>,
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

    let (free_tx, free_rx) = crossbeam::channel::unbounded::<u32>();
    for s in 0..slots {
        free_tx
            .send(u32::try_from(s).expect("slot count fits in u32"))
            .expect("receiver is live");
    }

    let stdin_reader = std::thread::spawn(move || {
        let mut stdin = std::io::stdin().lock();
        let mut buf = [0u8; 4];
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
            if let Err(e) = write_descriptor(
                &mut out,
                slot,
                u32::try_from(n).expect("n <= slot_size, a usize that fit a slot"),
            ) {
                if is_consumer_gone(&e) {
                    return Ok(());
                }
                return Err(e.into());
            }
            off += n;
        }
        if let Ok(v) = Arc::try_unwrap(chunk) {
            let _ = recycle_tx.try_send(v);
        }
    }
    producer.join().expect("producer thread panicked")?;

    if let Err(e) = write_descriptor(&mut out, u32::MAX, 0) {
        if is_consumer_gone(&e) {
            return Ok(());
        }
        return Err(e.into());
    }
    out.flush()?;
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
    _producer: std::thread::JoinHandle<Result<()>>,
) -> Result<()> {
    bail!("shared-memory output is only supported on Unix");
}

fn write_descriptor(out: &mut impl Write, slot: u32, len: u32) -> std::io::Result<()> {
    let mut desc = [0u8; 8];
    desc[0..4].copy_from_slice(&slot.to_le_bytes());
    desc[4..8].copy_from_slice(&len.to_le_bytes());
    out.write_all(&desc)?;
    out.flush()
}


fn is_consumer_gone(e: &std::io::Error) -> bool {
    e.kind() == std::io::ErrorKind::BrokenPipe
}
