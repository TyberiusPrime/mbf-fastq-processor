//! Shared helpers for the fastqrab integration tests.
//!
//! Lives in a `tests/` subdirectory so cargo does not compile it as its own test
//! target; each test binary that needs it pulls it in with
//! `#[path = "common/mod.rs"] mod common;`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// The generated-test harness (`run_test`), shared by `generated.rs`.
pub mod test_runner;

/// Build the `fastqrab-decompressor` binary once and return its path.
///
/// Cargo only guarantees the binaries of the package under test are built, and
/// `fastqrab-decompressor` is a separate, binary-only workspace crate that
/// `fastqrab` does not depend on — so `cargo test -p fastqrab` from a clean tree
/// would not build it, and any test case using `use_rapidgzip = true` would fail
/// to spawn it. We invoke `cargo build -p fastqrab-decompressor` and read the
/// produced binary's path straight out of cargo's JSON artifact stream (which
/// reports it even for an already-up-to-date "fresh" build, so we don't depend on
/// the top-level `target/<profile>/` hard-link existing). Callers hand that exact
/// path to the child via the `FASTQRAB_DECOMPRESSOR` env var, so tests pass
/// regardless of how they are invoked.
pub fn decompressor() -> &'static Path {
    static DECOMPRESSOR: OnceLock<PathBuf> = OnceLock::new();
    DECOMPRESSOR.get_or_init(|| {
        // `target/<profile>/fastqrab` ⇒ release build iff that dir is "release".
        let fastqrab = Path::new(env!("CARGO_BIN_EXE_fastqrab"));
        let release =
            fastqrab.parent().and_then(Path::file_name) == Some(std::ffi::OsStr::new("release"));

        let mut cmd = Command::new(env!("CARGO"));
        cmd.args([
            "build",
            "-p",
            "fastqrab-decompressor",
            "--message-format=json",
        ]);
        if release {
            cmd.arg("--release");
        }
        let output = cmd.output().expect("failed to invoke `cargo build`");
        assert!(
            output.status.success(),
            "cargo build -p fastqrab-decompressor failed:\n{}",
            String::from_utf8_lossy(&output.stderr),
        );

        // Find the compiler-artifact line for the decompressor binary and take its
        // `executable` path.
        let path = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .find_map(|msg| {
                (msg["reason"] == "compiler-artifact"
                    && msg["target"]["name"] == "fastqrab-decompressor")
                    .then(|| msg["executable"].as_str().map(PathBuf::from))
                    .flatten()
            })
            .expect("cargo build did not report a fastqrab-decompressor executable");
        assert!(
            path.exists(),
            "reported decompressor missing: {}",
            path.display()
        );
        path
    })
}
