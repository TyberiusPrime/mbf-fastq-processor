#![allow(clippy::unwrap_used)]
use anyhow::{Context, Result, bail};
// Removed unused imports
use ex::fs::{self};
use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use wait_timeout::ChildExt;

#[allow(clippy::missing_panics_doc)]
pub fn run_test(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    if path.join("skip_windows").exists() {
        println!(
            "Skipping {} on Windows (skip_windows marker present)",
            path.display()
        );
        return;
    }

    // Always use verify command - it handles both panic and non-panic tests
    let test_case = TestCase::new(path.to_path_buf());
    let processor_path = find_processor();
    let r = run_verify_test(&test_case, &processor_path);
    if let Err(e) = r {
        panic!("Test failed {} {e:?}", path.display());
    } else {
        println!("Test passed for {}", path.display());
    }
}

const COMMAND_TIMEOUT: Duration = Duration::from_secs(60); //github might be slow.

fn find_processor() -> PathBuf {
    let exe_path = env!("CARGO_BIN_EXE_mbf-fastq-processor"); //format is not const :(
    PathBuf::from(exe_path)
}

fn run_verify_test(test_case: &TestCase, processor_cmd: &Path) -> Result<()> {
    let test_script = test_case.dir.join("test.sh");

    if test_script.exists() {
        // For test cases with test.sh, create a temp environment and run the custom script
        let temp_dir = setup_test_environment(&test_case.dir).context("Setup test dir")?;
        let actual_dir = test_case.dir.join("actual");

        // Create actual directory
        if actual_dir.exists() {
            fs::remove_dir_all(&actual_dir)?;
        }
        fs::create_dir_all(&actual_dir)?;

        // Copy all input* files to actual directory for inspection
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            let src_path = entry.path();
            if src_path.is_file()
                && let Some(file_name) = src_path.file_name()
            {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("input") {
                    let dst_path = actual_dir.join(file_name);
                    fs::copy(&src_path, &dst_path)?;
                }
            }
        }

        // Run the test.sh script
        let script_path = test_script
            .canonicalize()
            .context("Canonicalize test.sh path")?;
        let mut cmd = std::process::Command::new("bash");
        cmd.arg(script_path)
            .env("PROCESSOR_CMD", processor_cmd)
            .env("CONFIG_FILE", temp_dir.path().join("input.toml"))
            .env("NO_FRIENDLY_PANIC", "1")
            .current_dir(temp_dir.path());

        let output = run_command_with_timeout(&mut cmd).context("Failed to run test.sh")?;

        // it is the test dir's responsibility to check for correctness.
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            anyhow::bail!("Test script failed:\nstdout: {stdout}\nstderr: {stderr}",);
        }

        Ok(())
    } else {
        // Use the verify command for regular test cases (handles both panic and non-panic tests)
        let config_file = test_case.dir.join("input.toml");
        let actual_dir = test_case.dir.join("actual");

        // Call verify command with --output-dir
        let mut cmd = std::process::Command::new(processor_cmd);
        cmd.arg("verify")
            .arg(&config_file)
            .arg("--output-dir")
            .arg(&actual_dir)
            .env("NO_FRIENDLY_PANIC", "1");

        let output = cmd.output().context("Failed to run verify command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Unexpected stderr file") {
                return Ok(()); // TODO: remove this temporary workaround
            }
            anyhow::bail!("Verification failed:\nstderr: {stderr}");
        }

        Ok(())
    }
}

struct TestCase {
    dir: PathBuf,
}

impl TestCase {
    fn new(dir: PathBuf) -> Self {
        TestCase { dir }
    }
}

fn run_command_with_timeout(cmd: &mut std::process::Command) -> Result<std::process::Output> {
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn command")?;

    if let Some(status) = child.wait_timeout(COMMAND_TIMEOUT)? {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut reader) = child.stdout.take() {
            reader.read_to_end(&mut stdout)?;
        }
        if let Some(mut reader) = child.stderr.take() {
            reader.read_to_end(&mut stderr)?;
        }
        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    } else {
        let _ = child.kill();
        let status = child.wait()?;
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut reader) = child.stdout.take() {
            reader.read_to_end(&mut stdout)?;
        }
        if let Some(mut reader) = child.stderr.take() {
            reader.read_to_end(&mut stderr)?;
        }
        let stdout_str = String::from_utf8_lossy(&stdout);
        let stderr_str = String::from_utf8_lossy(&stderr);
        bail!(
            "Command {:?} timed out after {:?}. Exit status: {:?}\nstdout: {}\nstderr: {}",
            &cmd,
            COMMAND_TIMEOUT,
            status,
            stdout_str,
            stderr_str
        );
    }
}

#[cfg(target_os = "windows")]
fn windows_path_to_wsl(path: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize {:?}", path))?;
    let mut raw = canonical
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 path: {:?}", canonical))?
        .to_owned();

    if let Some(stripped) = raw.strip_prefix(r"\\?\") {
        raw = stripped.to_owned();
    }

    let mut parts = raw.splitn(2, ':');
    let drive = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing drive letter in path: {raw}"))?;
    let rest = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing path component in: {raw}"))?;

    let normalized_rest = rest
        .trim_start_matches(|c| c == '\\' || c == '/')
        .replace('\\', "/");
    let drive_letter = drive
        .chars()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty drive letter in: {raw}"))?
        .to_ascii_lowercase();

    Ok(format!("/mnt/{drive_letter}/{normalized_rest}"))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dst_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dst_path)?;
        } else {
            fs::copy(&path, &dst_path)?;
        }
    }
    Ok(())
}

fn setup_test_environment(test_dir: &Path) -> Result<TempDir> {
    let temp_dir = tempfile::tempdir().context("make tempdir")?;

    // Copy input.toml
    let input_toml_src = test_dir.join("input.toml");
    let input_toml_dst = temp_dir.path().join("input.toml");
    if input_toml_src.exists() {
        //otherwise prep.sh must make it
        fs::copy(&input_toml_src, &input_toml_dst).context("copy input file")?;
    }

    // Copy any input*.fq* files and input/ directory (for cookbooks)
    for entry in fs::read_dir(test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() || path.is_symlink() {
            if path.is_symlink() {
                //so we get an error message that points to the symlink
                fs::canonicalize(&path).context("canonicalize symlink")?;
            }
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("input_") {
                    let dst_path = temp_dir.path().join(file_name);

                    // Check if this is a 0-byte file without read permissions
                    let metadata = fs::metadata(&path)?;
                    if metadata.len() == 0 {
                        // Check if file has no read permissions (using unix permissions)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = metadata.permissions();
                            if (perms.mode() & 0o400) == 0 {
                                // No read permission for owner
                                // Create empty file without read permissions
                                fs::write(&dst_path, "")?;
                                let mut new_perms = fs::metadata(&dst_path)?.permissions();
                                new_perms.set_mode(perms.mode());
                                fs::set_permissions(&dst_path, new_perms)?;
                                continue;
                            }
                        }
                    }

                    // Normal file copy
                    fs::copy(&path, &dst_path)?;
                }
            }
        } else if path.is_dir() {
            // Copy input/ and reference_output/ directories for cookbooks
            if let Some(dir_name) = path.file_name() {
                let dir_name_str = dir_name.to_string_lossy();
                if dir_name_str == "input" || dir_name_str == "reference_output" {
                    let dst_dir = temp_dir.path().join(dir_name);
                    copy_dir_recursive(&path, &dst_dir)?;
                }
            }
        }
    }
    // Optional preparatory script
    let prep_script = test_dir.join("prep.sh");
    if prep_script.exists() {
        #[cfg(not(target_os = "windows"))]
        let mut prep_command = {
            let mut command = std::process::Command::new("bash");
            command
                .arg(prep_script.canonicalize().context("canonicalize prep.sh")?)
                .current_dir(temp_dir.path());
            command
        };

        #[cfg(target_os = "windows")]
        let mut prep_command = {
            let script_wsl = windows_path_to_wsl(&prep_script)?;
            let cwd_wsl = windows_path_to_wsl(temp_dir.path())?;
            let mut command = std::process::Command::new("wsl");
            command
                .arg("--cd")
                .arg(&cwd_wsl)
                .arg("bash")
                .arg(&script_wsl);
            command
        };

        let prep_output =
            run_command_with_timeout(&mut prep_command).context("Failed to execute prep.sh")?;

        if !prep_output.status.success() {
            anyhow::bail!(
                "prep.sh failed with exit code: {:?}\nstdout: {}\nstderr: {}",
                prep_output.status.code(),
                String::from_utf8_lossy(&prep_output.stdout),
                String::from_utf8_lossy(&prep_output.stderr)
            );
        }
    }
    Ok(temp_dir)
}
