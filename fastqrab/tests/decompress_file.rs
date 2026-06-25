//! `fastqrab::decompress_file` decodes through the out-of-process decompressor
//! (the single gzip/zstd decode path), so it needs the sibling binary — which a
//! `--lib` unit test can't see. This integration test points
//! `FASTQRAB_DECOMPRESSOR` at the freshly built binary and round-trips gzip and
//! zstd (and an uncompressed passthrough).
#![expect(clippy::unwrap_used, reason = "it's tests")]

use std::io::Write as _;

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
