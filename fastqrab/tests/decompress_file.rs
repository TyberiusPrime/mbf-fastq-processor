//! `fastqrab::decompress_file` decodes through the out-of-process decompressor
//! (the single gzip/zstd decode path), so it needs the sibling binary — which a
//! `--lib` unit test can't see. This integration test points
//! `FASTQRAB_DECOMPRESSOR` at the freshly built binary and round-trips gzip and
//! zstd (and an uncompressed passthrough).

use std::io::Write as _;
use std::process::{Command, Stdio};

#[path = "common/mod.rs"]
mod common;

#[test]
fn decompress_file_roundtrips_gzip_zstd_and_plain() {
    // SAFETY: set once, before any decompressor spawn, in this single-threaded test.
    unsafe {
        std::env::set_var("FASTQRAB_DECOMPRESSOR", common::decompressor());
    }

    let payload: &[u8] = b"Hello, world!\nsecond line\n";
    let dir = tempfile::tempdir().unwrap();

    // gzip (gzp/standard gzip both decode via the subprocess).
    let gz = dir.path().join("f.gz");
    let mut enc = flate2::write::GzEncoder::new(
        std::fs::File::create(&gz).unwrap(),
        flate2::Compression::default(),
    );
    enc.write_all(payload).unwrap();
    enc.finish().unwrap();
    assert_eq!(fastqrab::decompress_file(&gz).unwrap(), payload);

    // zstd.
    let zst = dir.path().join("f.zst");
    std::fs::write(&zst, zstd::encode_all(payload, 3).unwrap()).unwrap();
    assert_eq!(fastqrab::decompress_file(&zst).unwrap(), payload);

    // uncompressed passthrough.
    let plain = dir.path().join("f.txt");
    std::fs::write(&plain, payload).unwrap();
    assert_eq!(fastqrab::decompress_file(&plain).unwrap(), payload);
}

/// Drives the `__decompressor` subcommand directly (bypassing
/// `fastqrab::decompress_file`, which never disconnects its own pipe) so we can
/// close the read end out from under it. `run_pipe`'s `write_all` to stdout must
/// then observe a broken pipe and return cleanly (`is_consumer_gone`) rather than
/// propagating an error or panicking.
#[test]
fn decompressor_exits_cleanly_when_consumer_closes_pipe() {
    let dir = tempfile::tempdir().unwrap();
    let gz = dir.path().join("f.gz");
    // Several decode chunks' worth of data, so the closed pipe is hit mid-stream
    // rather than only after a single write.
    let payload = vec![b'x'; 8 * 1024 * 1024];
    let mut enc = flate2::write::GzEncoder::new(
        std::fs::File::create(&gz).unwrap(),
        flate2::Compression::fast(),
    );
    enc.write_all(&payload).unwrap();
    enc.finish().unwrap();

    let mut child = Command::new(common::decompressor())
        .arg("__decompressor")
        .arg(&gz)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // Drop our end of the stdout pipe immediately: the child then holds the only
    // remaining writer with no reader left, so its next write to stdout fails
    // with a broken pipe.
    drop(child.stdout.take());

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "decompressor should treat a closed stdout pipe as a clean exit, got {:?}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr), // cov:excl-line
    );
}

/// Same idea as [`decompressor_exits_cleanly_when_consumer_closes_pipe`], but for
/// the shm transport: `--shm-fd` points the child at a real `memfd`-backed region
/// (mirroring `spawn_decompressor_shm`'s setup) and its descriptors go out over
/// stdout, same as the pipe path. Closing our end before the child ever writes a
/// descriptor forces the *first* `write_descriptor` call in `run_shm`'s per-chunk
/// loop to observe a broken pipe, hitting the early `is_consumer_gone` return
/// (decompressor.rs:364) rather than the later one guarding the EOF sentinel.
#[cfg(unix)]
#[test]
fn decompressor_shm_exits_cleanly_when_consumer_closes_pipe() {
    let dir = tempfile::tempdir().unwrap();
    let gz = dir.path().join("f.gz");
    let payload = vec![b'x'; 8 * 1024 * 1024];
    let mut enc = flate2::write::GzEncoder::new(
        std::fs::File::create(&gz).unwrap(),
        flate2::Compression::fast(),
    );
    enc.write_all(&payload).unwrap();
    enc.finish().unwrap();

    let slots = 8usize;
    let slot_size = 8 * 1024 * 1024usize;
    let total = slots * slot_size;

    // SAFETY: standard libc calls creating and sizing a fresh memfd, mirroring
    // `spawn_decompressor_shm`. No `MFD_CLOEXEC`, so the spawned child inherits
    // this fd (same number) for `--shm-fd`.
    let fd = unsafe { libc::memfd_create(c"fastqrab-shm-test".as_ptr(), 0) };
    assert!(
        fd >= 0,
        "memfd_create failed: {}",
        std::io::Error::last_os_error() // cov:excl-line
    );
    // SAFETY: `fd` is the memfd just created above.
    let rc = unsafe { libc::ftruncate(fd, libc::off_t::try_from(total).unwrap()) };
    assert_eq!(
        rc,
        0,
        "ftruncate failed: {}",
        std::io::Error::last_os_error() // cov:excl-line
    );

    let mut child = Command::new(common::decompressor())
        .arg("__decompressor")
        .arg("--format")
        .arg("gzip")
        .arg("--shm-fd")
        .arg(fd.to_string())
        .arg("--shm-slots")
        .arg(slots.to_string())
        .arg("--shm-slot-size")
        .arg(slot_size.to_string())
        .arg(&gz)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    // The child inherited its own reference to the memfd at fork; ours is unused
    // from here on.
    // SAFETY: `fd` is ours and no longer referenced in this process.
    unsafe { libc::close(fd) };

    // Drop our end of the descriptor pipe before the child ever writes a
    // `(slot, len)` descriptor, so the first `write_descriptor` call fails.
    drop(child.stdout.take());

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "shm decompressor should treat a closed descriptor pipe as a clean exit, got {:?}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr), // cov:excl-line
    );
}
