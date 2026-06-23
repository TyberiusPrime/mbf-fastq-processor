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

/// Lay out a self-contained test case (gz input, reference output, identity
/// pipeline toml) in `dir`, then run `fastqrab verify` under the given shm env
/// overrides. A success exit means the output matched the reference byte for
/// byte through the shm transport.
fn run_case(dir: &Path, payload: &[u8], env: &[(&str, &str)]) {
    std::fs::write(dir.join("input_read1.fq.gz"), gzip(payload)).unwrap();
    // Identity pipeline ⇒ the reference output is the input verbatim.
    std::fs::write(dir.join("output_read1.fq"), payload).unwrap();
    std::fs::write(
        dir.join("input.toml"),
        "\
[input]
    read1 = ['input_read1.fq.gz']

[input.options]
    use_rapidgzip = true

[options]
    block_size = 1000
    threads = 2

[output]
    prefix = \"output\"

[[step]]
    action = \"OutputFASTQ\"
",
    )
    .unwrap();

    let fastqrab = env!("CARGO_BIN_EXE_fastqrab");
    let mut cmd = Command::new(fastqrab);
    cmd.arg("verify")
        .arg(dir.join("input.toml"))
        .arg("--output-dir")
        .arg(dir.join("actual"))
        .env("NO_FRIENDLY_PANIC", "1");
    for (k, val) in env {
        cmd.env(k, val);
    }
    let output = cmd.output().expect("failed to run fastqrab verify");
    assert!(
        output.status.success(),
        "verify failed (env={env:?})\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
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
