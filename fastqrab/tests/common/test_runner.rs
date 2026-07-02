#![expect(clippy::unwrap_used, reason = "it's tests")]
#![allow(dead_code, reason = "not every test binary uses every helper")]
use anyhow::{Context, Result};
use std::env;
use std::path::{Path, PathBuf};

/// # Panics
/// When the test fails
pub fn run_test(path: &std::path::Path, toml_name: &str, test_no_in_directory: usize) {
    #[cfg(target_os = "windows")]
    if path.join("skip_windows").exists() {
        if path.join("should_panic").exists() {
            // Test is marked #[should_panic] in generated.rs; panic so it passes.
            panic!(
                "Skipping {} on Windows (skip_windows marker present)",
                path.display()
            );
        }
        println!(
            "Skipping {} on Windows (skip_windows marker present)",
            path.display()
        );
        return;
    }
    #[cfg(target_os = "windows")]
    if path.join("test.sh").exists()
        || path.join("prep.sh").exists()
        || path.join("post.sh").exists()
    {
        println!(
            "Skipping {} on Windows (shell scripts not supported)",
            path.display()
        );
        return;
    }
    #[cfg(target_os = "macos")]
    if path.join("skip_macos").exists() {
        println!(
            "Skipping {} on macOS (skip_macos marker present)",
            path.display()
        );
        return;
    }
    if env::var_os("GITHUB_ACTIONS").is_some() && path.join("skip_github").exists() {
        //cov:excl-start
        println!(
            "Skipping {} on GitHub Actions (skip_github marker present)",
            path.display()
        );
        return;
        //cov:excl-stop
    }

    // Always use verify command - it handles both panic and non-panic tests
    let measure_alloc = path.join("measure_alloc").exists();
    let processor_path = find_processor(measure_alloc);
    let r = run_verify_test(path, &processor_path, toml_name, test_no_in_directory);
    if let Err(e) = r {
        panic!("Test failed {} {e:?}", path.display());
    } else {
        println!("Test passed for {}", path.display());
    }
}

fn find_processor(measure_alloc: bool) -> PathBuf {
    if measure_alloc {
        let exe_path = env!("CARGO_BIN_EXE_fastqrab_alloc_accounting"); //format is not const :(
        PathBuf::from(exe_path)
    } else {
        let exe_path = env!("CARGO_BIN_EXE_fastqrab"); //format is not const :(
        PathBuf::from(exe_path)
    }
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
        // Trigger ~half with relative and ~half with absolute dir for path coverage
        // in verify.rs. Both branches must use paths that are correct regardless of
        // how resolve_paths() in verify.rs resolves them: absolute paths are used
        // as-is; relative paths are joined with toml_dir, so they must be relative
        // to toml_dir (i.e. just "actual"), not relative to the test-runner CWD.
        if test_case_dir
            .file_name()
            .and_then(|ostr| ostr.to_str())
            .and_then(|s| s.bytes().last())
            .is_some_and(|x| x & 1 == 1)
        {
            test_case_dir.canonicalize().unwrap().join("actual")
        } else {
            // "actual" is relative → resolve_paths joins it with toml_dir, landing
            // correctly at <test_case_dir>/actual.
            PathBuf::from("actual")
        }
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
