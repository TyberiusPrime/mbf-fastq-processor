//! End-to-end coverage of the rapidgzip *shared-memory* transport (the
//! out-of-process decompressor memcpying decode chunks into shared slots and the
//! pod parser consuming them in place).
//!
//! These drive the real `fastqrab` binary — so the sibling
//! `fastqrab-decompressor` is discovered next to it exactly as in production —
//! over a generated multi-MiB gzip FASTQ, and verify byte-exact output through
//! `fastqrab verify`. Beyond the small `test_cases/input/compression/rapidgzip`
//! fixtures, they exercise: many decode chunks (slot recycling + the EOF
//! sentinel), a slot far smaller than a chunk (the multi-slot chunk-split path),
//! and a two-slot ring (heavy backpressure / recycling with no deadlock).
//!
//! Unix only: the shm transport is Unix-only (`memfd` + `MAP_SHARED`).
#![cfg(unix)]
#![expect(clippy::unwrap_used, reason = "it's tests")]

use std::io::Write as _;
use std::path::Path;
use std::process::Command;

use flate2::Compression;
use flate2::write::GzEncoder;

#[path = "common/mod.rs"]
mod common;
use common::decompressor;

/// Build a multi-record FASTQ payload (~`reads` records, variable read lengths,
/// space-free names so output is a byte-exact identity roundtrip). Large enough
/// to span several decode chunks.
fn make_fastq(reads: usize) -> Vec<u8> {
    let mut v = Vec::new();
    for i in 0..reads {
        let len = 80 + (i % 137); // variable length: 80..=216
        // Pseudo-random but deterministic base/qual fill.
        let seq: Vec<u8> = (0..len)
            .map(|j| b"ACGT"[(i.wrapping_mul(31).wrapping_add(j)) % 4])
            .collect();
        let qual: Vec<u8> = (0..len)
            .map(|j| 33 + u8::try_from((i.wrapping_add(j * 7)) % 40).unwrap())
            .collect();
        write!(v, "@read{i:08}\n").unwrap();
        v.extend_from_slice(&seq);
        v.extend_from_slice(b"\n+\n");
        v.extend_from_slice(&qual);
        v.push(b'\n');
    }
    v
}

fn gzip(data: &[u8]) -> Vec<u8> {
    let mut enc = GzEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).unwrap();
    enc.finish().unwrap()
}

fn zstd_compress(data: &[u8]) -> Vec<u8> {
    zstd::encode_all(data, 3).unwrap()
}

/// Build a multi-record wrapped FASTA payload (descriptions on some headers,
/// sequence wrapped across multiple lines) to exercise the pod-FASTA parser's
/// header split and multi-line/cross-chunk sequence assembly.
fn make_fasta(records: usize) -> Vec<u8> {
    let mut v = Vec::new();
    for i in 0..records {
        if i % 3 == 0 {
            write!(v, ">seq{i:06} some description {i}\n").unwrap();
        } else {
            write!(v, ">seq{i:06}\n").unwrap();
        }
        let len = 60 + (i % 100);
        let seq: Vec<u8> = (0..len)
            .map(|j| b"ACGT"[(i.wrapping_mul(7).wrapping_add(j)) % 4])
            .collect();
        for line in seq.chunks(70) {
            v.extend_from_slice(line);
            v.push(b'\n');
        }
    }
    v
}

/// Lay out a self-contained identity test case (compressed input named
/// `input_name`, reference output, identity pipeline toml) in `dir`, then run
/// `fastqrab verify` under the given shm env overrides. A success exit means the
/// output matched the reference byte for byte through the transport under test.
fn run_verify_case(
    dir: &Path,
    input_name: &str,
    compressed: &[u8],
    plain: &[u8],
    env: &[(&str, &str)],
) {
    std::fs::write(dir.join(input_name), compressed).unwrap();
    // Identity pipeline ⇒ the reference output is the input verbatim.
    std::fs::write(dir.join("output_read1.fq"), plain).unwrap();
    std::fs::write(
        dir.join("input.toml"),
        format!(
            "\
[input]
    read1 = ['{input_name}']

[input.options]
    use_rapidgzip = true

[options]
    block_size = 1000
    threads = 2

[output]
    prefix = \"output\"

[[step]]
    action = \"OutputFASTQ\"
"
        ),
    )
    .unwrap();

    let fastqrab = env!("CARGO_BIN_EXE_fastqrab");
    let mut cmd = Command::new(fastqrab);
    cmd.arg("verify")
        .arg(dir.join("input.toml"))
        .arg("--output-dir")
        .arg(dir.join("actual"))
        .env("NO_FRIENDLY_PANIC", "1")
        .env("FASTQRAB_DECOMPRESSOR", decompressor());
    for (k, val) in env {
        cmd.env(k, val);
    }
    let output = cmd.output().expect("failed to run fastqrab verify");
    assert!(
        output.status.success(),
        "verify failed (input={input_name}, env={env:?})\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Gzip-FASTQ identity convenience over [`run_verify_case`].
fn run_case(dir: &Path, payload: &[u8], env: &[(&str, &str)]) {
    run_verify_case(dir, "input_read1.fq.gz", &gzip(payload), payload, env);
}

/// Run a FASTA→FASTQ pipeline over a `compressed` FASTA input and return the
/// produced `output_read1.fq` bytes. Used to compare the shm pod-FASTA parser
/// against the bio reader for byte-for-byte equality.
fn run_process_fasta(input_name: &str, compressed: &[u8], env: &[(&str, &str)]) -> Vec<u8> {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join(input_name), compressed).unwrap();
    std::fs::write(
        dir.join("input.toml"),
        format!(
            "\
[input]
    read1 = ['{input_name}']

[input.options]
    use_rapidgzip = true
    fasta_fake_quality = 'B'

[options]
    block_size = 1000
    threads = 2

[output]
    prefix = \"output\"

[[step]]
    action = \"OutputFASTQ\"
"
        ),
    )
    .unwrap();

    let fastqrab = env!("CARGO_BIN_EXE_fastqrab");
    let mut cmd = Command::new(fastqrab);
    cmd.arg("process")
        .arg("input.toml")
        .current_dir(dir)
        .env("NO_FRIENDLY_PANIC", "1")
        .env("FASTQRAB_DECOMPRESSOR", decompressor());
    for (k, val) in env {
        cmd.env(k, val);
    }
    let output = cmd.output().expect("failed to run fastqrab process");
    assert!(
        output.status.success(),
        "process failed (input={input_name}, env={env:?})\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    std::fs::read(dir.join("output_read1.fq")).expect("read FASTA pipeline output")
}

/// Default ring/slot config over a multi-MiB input: many decode chunks, slot
/// recycling, and the EOF sentinel.
#[test]
fn shm_large_multichunk_default_ring() {
    let payload = make_fastq(60_000); // ~9 MiB ⇒ multiple decode chunks
    let dir = tempfile::tempdir().unwrap();
    run_case(dir.path(), &payload, &[]);
}

/// A 64 KiB slot is far smaller than a ~4 MiB decode chunk, so every chunk is
/// split across ~64 slots: exercises the multi-slot chunk-split path and its
/// per-slot descriptors.
#[test]
fn shm_tiny_slots_force_chunk_split() {
    let payload = make_fastq(60_000);
    let dir = tempfile::tempdir().unwrap();
    run_case(
        dir.path(),
        &payload,
        &[("FASTQRAB_DECOMP_SHM_SLOT_SIZE", "65536")],
    );
}

/// A two-slot ring with tiny slots maximizes backpressure and recycling: the
/// decompressor constantly blocks for a returned slot. Must stay correct and
/// deadlock-free.
#[test]
fn shm_tiny_ring_backpressure() {
    let payload = make_fastq(60_000);
    let dir = tempfile::tempdir().unwrap();
    run_case(
        dir.path(),
        &payload,
        &[
            ("FASTQRAB_DECOMP_SHM_SLOTS", "2"),
            ("FASTQRAB_DECOMP_SHM_SLOT_SIZE", "65536"),
        ],
    );
}

/// The `FASTQRAB_DECOMP_SHM=0` escape hatch forces the legacy pipe transport;
/// output must be identical to the shm path.
#[test]
fn shm_disabled_falls_back_to_pipe() {
    let payload = make_fastq(20_000);
    let dir = tempfile::tempdir().unwrap();
    run_case(dir.path(), &payload, &[("FASTQRAB_DECOMP_SHM", "0")]);
}

/// zstd FASTQ over the shared-memory transport: the decompressor decodes with
/// libzstd (`--format zstd`) and memcpies into slots exactly as the gzip path, so
/// a multi-chunk identity roundtrip must reproduce the input byte-for-byte.
#[test]
fn shm_zstd_fastq_roundtrip() {
    let payload = make_fastq(60_000);
    let dir = tempfile::tempdir().unwrap();
    run_verify_case(
        dir.path(),
        "input_read1.fq.zst",
        &zstd_compress(&payload),
        &payload,
        &[],
    );
}

/// zstd FASTQ with tiny slots forces the multi-slot chunk-split path on the zstd
/// producer (whose chunks come from the streaming decode loop, not rapidgzip).
#[test]
fn shm_zstd_tiny_slots_force_chunk_split() {
    let payload = make_fastq(60_000);
    let dir = tempfile::tempdir().unwrap();
    run_verify_case(
        dir.path(),
        "input_read1.fq.zst",
        &zstd_compress(&payload),
        &payload,
        &[("FASTQRAB_DECOMP_SHM_SLOT_SIZE", "65536")],
    );
}

/// gzip FASTA over shm (the pod-FASTA parser) must produce output byte-identical
/// to the bio reader (forced via `FASTQRAB_DECOMP_SHM=0`). This pins the new
/// in-place FASTA assembler — header split, multi-line and cross-chunk sequence
/// concatenation, faked quality — to rust-bio's behavior.
#[test]
fn shm_fasta_gzip_matches_bio() {
    let fasta = make_fasta(20_000);
    let gz = gzip(&fasta);
    let shm = run_process_fasta("input.fa.gz", &gz, &[]);
    let bio = run_process_fasta("input.fa.gz", &gz, &[("FASTQRAB_DECOMP_SHM", "0")]);
    assert!(!shm.is_empty(), "FASTA pipeline produced no output");
    assert_eq!(shm, bio, "shm pod-FASTA output must match the bio reader");
}

/// As above but zstd-compressed: exercises zstd → FASTA shm decode end to end.
#[test]
fn shm_fasta_zstd_matches_bio() {
    let fasta = make_fasta(15_000);
    let zst = zstd_compress(&fasta);
    let shm = run_process_fasta("input.fa.zst", &zst, &[]);
    let bio = run_process_fasta("input.fa.zst", &zst, &[("FASTQRAB_DECOMP_SHM", "0")]);
    assert!(!shm.is_empty(), "FASTA pipeline produced no output");
    assert_eq!(
        shm, bio,
        "shm pod-FASTA (zstd) output must match the bio reader"
    );
}

/// Tiny slots over zstd FASTA: forces cross-slot/cross-chunk sequence assembly in
/// the pod-FASTA parser (a record's wrapped sequence straddling many slots).
#[test]
fn shm_fasta_zstd_tiny_slots() {
    let fasta = make_fasta(15_000);
    let zst = zstd_compress(&fasta);
    let shm = run_process_fasta(
        "input.fa.zst",
        &zst,
        &[("FASTQRAB_DECOMP_SHM_SLOT_SIZE", "65536")],
    );
    let bio = run_process_fasta("input.fa.zst", &zst, &[("FASTQRAB_DECOMP_SHM", "0")]);
    assert_eq!(shm, bio, "shm pod-FASTA must be chunk-boundary independent");
}
