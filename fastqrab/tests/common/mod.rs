#![expect(
    clippy::allow_attributes,
    reason = "dead code in some tests, but not all test binaries"
)] //we need an allow since it triggers only in some test binaryies
//! Shared helpers for the fastqrab integration tests.
//!
//! Lives in a `tests/` subdirectory so cargo does not compile it as its own test
//! target; each test binary that needs it pulls it in with
//! `#[path = "common/mod.rs"] mod common;`.

use std::path::Path;

/// The generated-test harness (`run_test`), shared by `generated.rs`.
pub mod test_runner;

/// Return the fastqrab binary path for use as the decompressor subprocess.
///
/// The decompressor is now a subcommand of the main fastqrab binary
/// (`fastqrab __decompressor …`), so we just return the already-built
/// fastqrab binary that cargo made available via `CARGO_BIN_EXE_fastqrab`.
/// Callers hand that path to the child via the `FASTQRAB_DECOMPRESSOR` env var.
///
/// (the tests need this since they otherwise try to call their own binary!
#[allow(
    dead_code,
    reason = " not all test binaries that include this module use it"
)]
pub fn decompressor() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_fastqrab"))
}
