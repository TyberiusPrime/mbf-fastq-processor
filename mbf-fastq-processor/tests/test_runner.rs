#![allow(clippy::unwrap_used)]
use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

#[allow(clippy::missing_panics_doc)]
pub fn run_test(path: &std::path::Path, toml_name: &str, test_no_in_directory: usize) {
    #[cfg(target_os = "windows")]
    if path.join("skip_windows").exists() {
        println!(
            "Skipping {} on Windows (skip_windows marker present)",
            path.display()
        );
        return;
    }

    // Always use verify command - it handles both panic and non-panic tests
    let processor_path = find_processor();
    let r = run_verify_test(&path, &processor_path, toml_name, test_no_in_directory);
    if let Err(e) = r {
        panic!("Test failed {} {e:?}", path.display());
    } else {
        println!("Test passed for {}", path.display());
    }
}

fn find_processor() -> PathBuf {
    let exe_path = env!("CARGO_BIN_EXE_mbf-fastq-processor"); //format is not const :(
    PathBuf::from(exe_path)
}

fn run_verify_test(
    test_case_dir: &Path,
    processor_cmd: &Path,
    toml_name: &str,
    test_no_in_directory: usize,
) -> Result<()> {
    let actual_dir = if test_no_in_directory > 1 {
        test_case_dir
            .canonicalize()
            .unwrap()
            .join(format!("actual_{test_no_in_directory}"))
    } else {
        test_case_dir.canonicalize().unwrap().join("actual")
    };

    // Use the verify command for regular test cases (handles both panic and non-panic tests)
    let config_file = test_case_dir.join(toml_name);
    let prep_file = test_case_dir.join("prep.sh");
    let post_file = test_case_dir.join("post.sh");
    let test_file = test_case_dir.join("test.sh");

    // Call verify command with --output-dir
    let mut cmd = std::process::Command::new(processor_cmd);
    cmd.arg("verify")
        .arg(&config_file)
        .arg("--output-dir")
        .arg(&actual_dir)
        .env("NO_FRIENDLY_PANIC", "1");
    if prep_file.exists() || post_file.exists() || test_file.exists() {
        cmd.arg("--unsafe-call-prep-sh");
    }

    let output = cmd.output().context("Failed to run verify command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Verification failed:\nstderr: {stderr}");
    }

    Ok(())
}
