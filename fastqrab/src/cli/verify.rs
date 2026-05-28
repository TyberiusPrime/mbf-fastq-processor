use anyhow::{Context, Result, bail};
use ex::fs;
use regex::Regex;
use std::borrow::Cow;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use fastqrab_io::STDIN_MAGIC_PATH;

pub fn verify_outputs(
    toml_file: &Path,
    output_dir: Option<&Path>,
    unsafe_prep: bool,
) -> Result<()> {
    let (toml_dir, output_dir) = resolve_paths(toml_file, output_dir)?;

    let prep_script = toml_dir.join("prep.sh");
    let post_script = toml_dir.join("post.sh");
    let test_script = toml_dir.join("test.sh");
    // Copy (not symlink) when scripts may run in the temp dir and could follow symlinks to
    // mutate the original files (e.g. via chmod/touch on what appears to be a local file).
    let do_copy_input_files =
        toml_dir.join("copy_input").exists() || test_script.exists() || prep_script.exists();

    let (expected_validation_error, expected_validation_warning, expected_runtime_error) =
        load_expected_failures(&toml_dir)?;
    let expected_failure = {
        let validation_error = expected_validation_error.as_ref();
        let runtime_error = expected_runtime_error.as_ref();
        match (validation_error, runtime_error) {
            (Some(x), None) | (None, Some(x)) => Some(x),
            (None, None) => None,
            (Some(_), Some(_)) =>
            // cov:excl-start
            {
                unreachable!()
            } // cov:excl-stop
        }
    };

    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    let (_temp_dir, temp_path) = create_working_dir(output_dir.as_deref())?;
    let temp_toml_path = temp_path.join("config.toml");

    populate_working_dir(
        toml_file,
        &raw_config,
        &toml_dir,
        &temp_path,
        do_copy_input_files,
    )?; // cov:excl-line 

    run_prep_if_needed(
        &prep_script,
        &post_script,
        &test_script,
        &toml_dir,
        &temp_path,
        unsafe_prep,
    )?;

    // Change CWD to temp_path so that in-process config validation (which calls
    // tpd_from_toml → PartialBarcodes::verify → load_from_file) resolves relative
    // filenames like `input_reference.fa` against the working directory that
    // contains the symlinked inputs — the same directory the processing subprocess
    // uses via `.current_dir(temp_path)`.
    std::env::set_current_dir(&temp_path)
        .with_context(|| format!("Failed to set working directory to {}", temp_path.display()))?;

    validate_config_if_needed(
        &temp_toml_path,
        expected_validation_error.as_ref(),
        expected_runtime_error.is_some(),
        expected_validation_warning.as_ref(),
    )?;
    let (output_prefix, uses_stdout) = extract_output_config(&raw_config)?;

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let needs_alloc_measurement = toml_dir.join("measure_alloc").exists();
    if needs_alloc_measurement
        && current_exe.file_name().and_then(|n| n.to_str()) != Some("fastqrab_alloc_accounting")
    {
        bail!(
            "measure_alloc file found in {} but current executable is not the allocation-measuring variant. \
            To perform allocation measurement tests,\
            use the fastqrab_alloc_accounting binary.",
            toml_dir.display()
        );
    }
    let stdin_config = toml_dir.join("stdin_config").exists();
    let stdin_file = if stdin_config {
        Some(temp_toml_path.clone())
    } else {
        detect_stdin_file(&raw_config, &toml_dir)
    };

    if test_script.exists() {
        run_test_script_and_check(&test_script, &temp_path, &current_exe)?;
    } else {
        run_processor_and_verify(
            &current_exe,
            expected_failure,
            expected_validation_error.as_ref(),
            stdin_file,
            stdin_config,
            &temp_path,
            &temp_toml_path,
            uses_stdout,
            &output_prefix,
            &toml_dir,
            &post_script,
            unsafe_prep,
        )?;
    }

    cleanup_output_dir(output_dir.as_deref())?;
    Ok(())
}

fn resolve_paths(
    toml_file: &Path,
    output_dir: Option<&Path>,
) -> Result<(PathBuf, Option<PathBuf>)> {
    let toml_file_abs = toml_file.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize TOML file path: {}",
            toml_file.display()
        )
    })?;
    let toml_dir = toml_file_abs
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let resolved_output_dir = output_dir.map(|d| {
        if d.is_absolute() {
            d.to_owned()
        } else {
            toml_dir.join(d)
        }
    });
    Ok((toml_dir, resolved_output_dir))
}

fn load_expected_failures(
    toml_dir: &Path,
) -> Result<(
    Option<ExpectedFailure>,
    Option<ExpectedFailure>,
    Option<ExpectedFailure>,
)> {
    let expected_validation_error = ExpectedFailure::new(toml_dir, "error")?;
    let expected_validation_warning = ExpectedFailure::new(toml_dir, "validation_warning")?;
    let expected_runtime_error = ExpectedFailure::new(toml_dir, "runtime_error")?;

    let error_file_count =
        u8::from(expected_validation_error.is_some()) + u8::from(expected_runtime_error.is_some());
    if error_file_count > 1 {
        bail!(
            "Both expected_error(.txt|.regex) and expected_runtime_error(.txt|.regex) files exist. Please provide only one, depending on wether it's a validation or a processing error."
        );
    }

    Ok((
        expected_validation_error,
        expected_validation_warning,
        expected_runtime_error,
    ))
}

fn extract_output_config(raw_config: &str) -> Result<(String, bool)> {
    let result = crate::config::config_from_string(raw_config);

    if let Ok(parsed) = &result
        && let Some(benchmark) = &parsed.benchmark
        && benchmark.enable
    {
        bail!(
            "This is a benchmarking configuration, which can't be verified for it's output (it has none). Maybe turn off benchmark.enable in your TOML, or use another configuration?"
        )
    }

    Ok(result
        .ok()
        .and_then(|parsed| parsed.output.as_ref().map(|o| (o.prefix.clone(), o.stdout)))
        .unwrap_or_else(|| ("missing_output_config".to_string(), false)))
}

fn create_working_dir(output_dir: Option<&Path>) -> Result<(tempfile::TempDir, PathBuf)> {
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = if let Some(output_dir) = output_dir {
        if output_dir.exists() {
            // cov:excl-start
            cleanup_output_dir(Some(output_dir))?;
            // cov:excl-stop
        }
        std::fs::create_dir_all(output_dir).with_context(|| {
            // cov:excl-start
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
        // cov:excl-stop
        output_dir
            .canonicalize()
            .expect("Failed to canonicalize output dir")
    } else {
        temp_dir.path().to_owned()
    };
    Ok((temp_dir, temp_path))
}

fn populate_working_dir(
    toml_file: &Path,
    raw_config: &str,
    toml_dir: &Path,
    temp_path: &Path,
    do_copy_input_files: bool,
) -> Result<()> {
    // Copy the original TOML without modification
    ex::fs::copy(toml_file, temp_path.join("config.toml"))
        .context("Failed to copy TOML to temp directory")?;

    // Set up input files in the temp dir. When prep/test scripts will run (do_copy_input_files),
    // we must copy files so those scripts cannot mutate the originals (e.g. via chmod).
    // Otherwise, symlinks suffice — the TOML is kept verbatim so relative paths still resolve.
    // If the TOML is syntactically invalid (e.g. duplicate key), parsing fails here but we must
    // not propagate that error — the subprocess will report it in the proper formatted form.
    let toml_value: Option<toml::Value> = toml::from_str(raw_config).ok();

    if do_copy_input_files {
        copy_input_files(toml_dir, temp_path)?;
    } else {
        symlink_input_files(toml_value.as_ref(), toml_dir, temp_path)?;
    }
    Ok(())
}

fn copy_input_files(toml_dir: &Path, temp_path: &Path) -> Result<()> {
    for entry in fs::read_dir(toml_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path.is_file()
            && let Some(file_name) = src_path.file_name()
        {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with("input") && file_name_str != "input.toml" {
                let dst_path = temp_path.join(file_name);
                if !dst_path.exists() {
                    ex::fs::copy(&src_path, &dst_path)?;
                } // cov:excl-line
            }
        }
    }
    Ok(())
}

fn symlink_input_files(
    toml_value: Option<&toml::Value>,
    toml_dir: &Path,
    temp_path: &Path,
) -> Result<()> {
    // No prep/test scripts — safe to use symlinks so the TOML stays verbatim.
    // toml_value is None when the config is syntactically invalid; skip TOML-guided symlinks in
    // that case and fall through to the directory scan so the subprocess can report the error.
    if let Some(toml_value) = toml_value {
        if let Some(input_table) = toml_value.get("input").and_then(|v| v.as_table()) {
            for field_name in input_table.keys() {
                if field_name == "interleaved" || field_name == "options" {
                    continue;
                }
                if let Some(value) = input_table.get(field_name) {
                    create_symlinks_for_files(value, toml_dir, temp_path)?;
                } // cov:excl-line
            }
        }
        if let Some(steps) = toml_value.get("step").and_then(|v| v.as_array()) {
            for step in steps {
                if let Some(step_table) = step.as_table() {
                    for filename_key in ["filename", "filenames", "files", "reference"] {
                        if let Some(value) = step_table.get(filename_key) {
                            create_symlinks_for_files(value, toml_dir, temp_path)?;
                        }
                    }
                } // cov:excl-line
            }
        }
    }
    // Also symlink any ancillary input files (e.g. .bai index alongside .bam)
    // that aren't explicitly named in the TOML but live next to the inputs.
    for entry in fs::read_dir(toml_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path.is_file()
            && let Some(file_name) = src_path.file_name()
        {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with("input") && file_name_str != "input.toml" {
                let dst_path = temp_path.join(file_name);
                if std::fs::symlink_metadata(&dst_path).is_err() {
                    create_symlink(&src_path, &dst_path)?;
                }
            }
        }
    }
    Ok(())
}

#[allow(unused_variables)] // temp_path not used on windows
fn run_prep_if_needed(
    prep_script: &Path,
    post_script: &Path,
    test_script: &Path,
    toml_dir: &Path,
    temp_path: &Path,
    unsafe_prep: bool,
) -> Result<()> {
    if unsafe_prep {
        if prep_script.exists() {
            #[cfg(target_os = "windows")]
            {
                bail!("prep.sh execution on Windows is not currently supported");
            };

            #[cfg(not(target_os = "windows"))]
            {
                let mut prep_command = {
                    let mut command = std::process::Command::new("bash");
                    command
                        .arg(prep_script.canonicalize().context("canonicalize prep.sh")?)
                        .current_dir(temp_path);
                    command
                };

                let prep_output = prep_command.output().context("Failed to execute prep.sh")?;
                if !prep_output.status.success() {
                    bail!(
                        "prep.sh failed with exit code: {:?}\nstdout: {}\nstderr: {}",
                        prep_output.status.code(),
                        String::from_utf8_lossy(&prep_output.stdout),
                        String::from_utf8_lossy(&prep_output.stderr)
                    );
                }
            }
        }
    } else if prep_script.exists() {
        bail!(
            "prep.sh script found in {} but unsafe_prep is false. To enable prep.sh execution, pass in --unsafe-call-prep-sh on the CLI",
            toml_dir.display()
        );
    } else if post_script.exists() {
        bail!(
            "post.sh script found in {} but unsafe_prep is false. To enable post.sh execution, pass in --unsafe-call-prep-sh on the CLI",
            toml_dir.display()
        );
    } else if test_script.exists() {
        bail!(
            "test.sh script found in {} but unsafe_prep is false. To enable test.sh execution, pass in --unsafe-call-prep-sh on the CLI",
            toml_dir.display()
        );
    }
    Ok(())
}

fn validate_config_if_needed(
    temp_toml_path: &Path,
    expected_validation_error: Option<&ExpectedFailure>,
    has_runtime_error: bool,
    expected_validation_warning: Option<&ExpectedFailure>,
) -> Result<()> {
    if expected_validation_error.is_none() || expected_validation_warning.is_some() {
        let warnings =
            crate::cli::validate::validate_config(temp_toml_path).with_context(|| {
                if has_runtime_error {
                    "Configuration validation failed, but a runtime error was expected.".to_string()
                } else {
                    "Configuration validation failed unexpectedly.".to_string()
                }
            })?;
        if let Some(expected_warning) = expected_validation_warning {
            if warnings.is_empty() {
                bail!("Expected validation warning, but none were produced.");
            } else if !warnings.iter().any(|w| {
                expected_warning
                    .validate_expected_failure(w, temp_toml_path)
                    .is_ok()
            }) {
                bail!(
                    "Validation warnings did not match expected pattern.\nExpected: {}\nActual warnings:\n{}",
                    expected_warning,
                    warnings.join("\n")
                );
            }
        }
    }
    Ok(())
}

fn detect_stdin_file(raw_config: &str, toml_dir: &Path) -> Option<PathBuf> {
    if raw_config.contains(STDIN_MAGIC_PATH) {
        let stdin_path = toml_dir.join("stdin");
        if stdin_path.exists() {
            return Some(stdin_path);
        }
    }
    None
}

fn run_test_script_and_check(
    test_script: &Path,
    temp_path: &Path,
    current_exe: &Path,
) -> Result<()> {
    let mut command = std::process::Command::new("bash");
    command
        .arg(test_script)
        .env("PROCESSOR_CMD", current_exe)
        .env("CONFIG_FILE", "config.toml")
        .env("NO_FRIENDLY_PANIC", "1")
        .current_dir(temp_path);

    let output = run_command_with_timeout(&mut command).context("Failed to run test.sh")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("Test script failed:\nstdout: {stdout}\nstderr: {stderr}",);
    }
    Ok(())
}

fn execute_processor(
    command: &mut std::process::Command,
    stdin_file: Option<PathBuf>,
) -> Result<std::process::Output> {
    if let Some(stdin_path) = stdin_file {
        let stdin_content = ex::fs::read(&stdin_path)
            .with_context(|| format!("Failed to read stdin file: {}", stdin_path.display()))?;

        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn fastqrab subprocess")?;

        // Write stdin in a separate thread to avoid deadlock when the subprocess
        // fills its stdout/stderr buffers before consuming all stdin, or exits
        // early (e.g., on error), causing EPIPE on the write side.
        let mut stdin = child.stdin.take().expect("stdin is piped");
        let stdin_thread = std::thread::spawn(move || {
            let _ = stdin.write_all(&stdin_content); // ignore EPIPE if process exits early
        });

        let output = child
            .wait_with_output()
            .context("Failed to wait for subprocess completion")?;
        let _ = stdin_thread.join();
        Ok(output)
    } else {
        command
            .output()
            .context("Failed to execute fastqrab subprocess")
    }
}

fn verify_processor_success(
    output: &std::process::Output,
    temp_path: &Path,
    expected_dir: &Path,
    output_prefix: &str,
    uses_stdout: bool,
    post_script: &Path,
    unsafe_prep: bool,
) -> Result<()> {
    if !output.stdout.is_empty() {
        ex::fs::write(temp_path.join("stdout"), &output.stdout)
            .context("Failed to write stdout to temp directory")?;
    }
    if !output.stderr.is_empty() {
        // cov:excl-start
        ex::fs::write(temp_path.join("stderr"), &output.stderr)
            .context("Failed to write stderr to temp directory")?;
        // cov:excl-stop
    }

    let mut mismatches = Vec::new();

    if post_script.exists() && unsafe_prep {
        #[cfg(target_os = "windows")]
        {
            bail!("post.sh execution on Windows is not currently supported");
        };

        #[cfg(not(target_os = "windows"))]
        {
            let mut post_command = {
                let mut command = std::process::Command::new("bash");
                command
                    .arg(post_script.canonicalize().context("canonicalize post.sh")?)
                    .current_dir(temp_path);
                command
            };

            let post_output = post_command.output().context("Failed to execute post.sh")?;
            if !post_output.status.success() {
                mismatches.push(format!(
                    "post.sh failed with exit code: {:?}\nstdout: {}\nstderr: {}",
                    post_output.status.code(),
                    String::from_utf8_lossy(&post_output.stdout),
                    String::from_utf8_lossy(&post_output.stderr)
                ));
            }
        }
    }

    let actual_dir = temp_path;

    if !uses_stdout {
        let expected_files = find_output_files(expected_dir, output_prefix).unwrap_or_default();
        let has_expected_stream =
            expected_dir.join("stdout").exists() || expected_dir.join("stderr").exists();
        if expected_files.is_empty() && !has_expected_stream {
            bail!(
                "No expected output files found in {} with prefix '{}'",
                expected_dir.display(),
                output_prefix
            );
        }
        for expected_file in &expected_files {
            let surplus = expected_file
                .strip_prefix(expected_dir)
                .expect("Stripping the dir again should work...");
            let str_surplus = surplus.to_string_lossy();
            let actual_file = actual_dir.join(str_surplus.as_ref());
            if !actual_file.exists() {
                mismatches.push(format!("Missing output file: {str_surplus}"));
                continue;
            }
            if let Err(e) = compare_files(expected_file, &actual_file, expected_dir) {
                mismatches.push(format!("{str_surplus}: {e}"));
            }
        }
    }

    for stream_name in ["stdout", "stderr"] {
        let expected_stream_file = expected_dir.join(stream_name);
        let actual_stream_file = actual_dir.join(stream_name);
        if expected_stream_file.exists() {
            if !actual_stream_file.exists() {
                mismatches.push(format!("Missing {stream_name} file"));
            } else if let Err(e) =
                compare_files(&expected_stream_file, &actual_stream_file, expected_dir)
            {
                mismatches.push(format!("{stream_name}: {e}"));
            }
        } else if actual_stream_file.exists() {
            mismatches.push(format!("Unexpected {stream_name} file"));
        }
    }

    if !uses_stdout {
        let actual_files = find_output_files(actual_dir, output_prefix)?;
        for actual_file in &actual_files {
            let surplus = actual_file
                .strip_prefix(actual_dir)
                .expect("Stripping the dir again should work...");
            let str_surplus = surplus.to_string_lossy();
            let expected_file = expected_dir.join(str_surplus.as_ref());
            if !expected_file.exists() {
                mismatches.push(format!("Unexpected output file: {str_surplus}"));
            }
        }
    }

    if !mismatches.is_empty() {
        bail!("Output verification failed:\n  {}", mismatches.join("\n  "));
    }
    Ok(())
}

#[expect(clippy::too_many_arguments, reason = "We need them")]
fn run_processor_and_verify(
    current_exe: &Path,
    expected_failure: Option<&ExpectedFailure>,
    expected_validation_error: Option<&ExpectedFailure>,
    stdin_file: Option<PathBuf>,
    stdin_config: bool,
    temp_path: &Path,
    temp_toml_path: &Path,
    uses_stdout: bool,
    output_prefix: &str,
    toml_dir: &Path,
    post_script: &Path,
    unsafe_prep: bool,
) -> Result<()> {
    let mut command = std::process::Command::new(current_exe);
    command
        .arg(if expected_validation_error.is_none() {
            "process"
        } else {
            "validate"
        })
        .arg(if stdin_config { "-" } else { "config.toml" })
        .current_dir(temp_path);

    let output = execute_processor(&mut command, stdin_file)?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    match (expected_failure, output.status.success()) {
        (Some(expected_failure_pattern), false) => {
            if !output.stderr.is_empty() {
                ex::fs::write(temp_path.join("stderr"), &output.stderr)
                    .context("Failed to write stderr to temp directory")?;
            } // cov:excl-line
            expected_failure_pattern.validate_expected_failure(&stderr, temp_toml_path)?;
        }
        (Some(_), true) => {
            if expected_validation_error.is_some() {
                bail!(
                    "Expected validation failure but 'validate' command succeeded. stderr: {stderr}"
                );
            } else {
                bail!("Expected runtime failure but 'process' command succeeded. stderr: {stderr}");
            }
        }
        (None, false) => {
            bail!(
                "Processing failed with exit code {:?}. stderr: {}",
                output.status.code(),
                stderr
            );
        }
        (None, true) => {
            verify_processor_success(
                &output,
                temp_path,
                toml_dir,
                output_prefix,
                uses_stdout,
                post_script,
                unsafe_prep,
            )?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn cleanup_output_dir(output_dir: Option<&Path>) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if let Some(output_dir) = output_dir
        && output_dir.exists()
        && let Err(_) = ex::fs::remove_dir_all(output_dir)
    {
        //try chmod it to write+executable
        let _ = ex::fs::set_permissions(output_dir, std::fs::Permissions::from_mode(0o755));
        //also chmod +x all subdirs...
        for entry in ex::fs::read_dir(output_dir)?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = ex::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
            } // cov:excl-line
        }
        ex::fs::remove_dir_all(output_dir).with_context(|| {
            // cov:excl-start
            format!(
                "Failed to remove existing output directory: {}",
                output_dir.display()
            )
        })?;
        // cov:excl-stop
    }

    Ok(())
}
#[cfg(windows)]
fn cleanup_output_dir(output_dir: Option<&Path>) -> Result<()> {
    if let Some(output_dir) = output_dir
        && output_dir.exists()
    {
        ex::fs::remove_dir_all(output_dir).with_context(|| {
            // cov:excl-start
            format!(
                "Failed to remove existing output directory: {}",
                output_dir.display()
            )
        })?;
        // cov:excl-stop
    }

    Ok(())
}

pub(crate) fn compare_files(expected: &Path, actual: &Path, input_dir: &Path) -> Result<()> {
    let is_compressed = is_compressed_file(expected);

    let (expected_bytes, actual_bytes) = if is_compressed {
        let expected_uncompressed = decompress_file(expected)?;
        let actual_uncompressed = decompress_file(actual)?;

        let expected_compressed_size = std::fs::metadata(expected)?.len();
        let actual_compressed_size = std::fs::metadata(actual)?.len();

        let size_diff_percent =
            calculate_size_difference_percent(expected_compressed_size, actual_compressed_size);

        if size_diff_percent > 5.0 {
            bail!(
                "Compressed file size difference too large: expected {expected_compressed_size} bytes, got {actual_compressed_size} bytes ({size_diff_percent}% difference)",
            );
        }

        (expected_uncompressed, actual_uncompressed)
    } else {
        let expected_bytes = std::fs::read(expected)
            .with_context(|| format!("Failed to read expected file: {}", expected.display()))?;
        let actual_bytes = std::fs::read(actual)
            .with_context(|| format!("Failed to read actual file: {}", actual.display()))?;
        (expected_bytes, actual_bytes)
    };

    let (expected_normalized, actual_normalized) = if expected
        .extension()
        .is_some_and(|ext| ext == "json" || ext == "html" || ext == "progress")
    {
        //println!("applying normalization to {}", expected.display());
        let expected_str = String::from_utf8_lossy(&expected_bytes);
        let actual_str = String::from_utf8_lossy(&actual_bytes);

        let (expected_normalized, actual_normalized) =
            if expected.extension().is_some_and(|ext| ext == "progress") {
                let res = (
                    normalize_progress_content(&expected_str),
                    normalize_progress_content(&actual_str),
                );
                std::fs::write(actual, &res.1).with_context(|| {
                    // cov:excl-start
                    format!(
                        "Failed to write normalized actual report file: {}",
                        actual.display()
                    )
                })?;
                // cov:excl-stop
                res
            } else {
                let res = (
                    normalize_report_content(&expected_str, None),
                    normalize_report_content(&actual_str, Some(input_dir)),
                );
                std::fs::write(actual, &res.1).with_context(|| {
                    // cov:excl-start
                    format!(
                        "Failed to write normalized actual report file: {}",
                        actual.display()
                    )
                })?;
                // cov:excl-stop
                res
            };

        if expected_normalized.is_empty() {
            // cov:excl-start
            unreachable!("expected file was empty after normalization - shouldn't be? Bug");
            // cov:excl-stop
        }
        (
            expected_normalized.into_bytes(),
            actual_normalized.into_bytes(),
        )
    } else {
        (expected_bytes, actual_bytes)
    };

    if expected_normalized.len() != actual_normalized.len() {
        bail!(
            "File size mismatch: expected {} bytes, got {} bytes",
            expected_normalized.len(),
            actual_normalized.len()
        );
    }

    if expected_normalized != actual_normalized {
        for (i, (exp, act)) in expected_normalized
            .iter()
            .zip(actual_normalized.iter())
            .enumerate()
        {
            if exp != act {
                bail!("Content mismatch at byte {i}: expected 0x{exp:02x}, got 0x{act:02x}",);
            }
        }
        // cov:excl-start
        unreachable!("Content mismatch (no specific byte difference found)");
        // cov:excl-stop
    }

    Ok(())
}

/// # Panics
/// When the internal regexps get broken
#[must_use]
pub fn normalize_report_content(content: &str, input_dir: Option<&Path>) -> String {
    let normalize_re = Regex::new(
        r#""(?P<key>version|program_version|cwd|working_directory|repository)"\s*:\s*"[^"]*""#,
    )
    .expect("invalid normalize regex");

    let content = normalize_re
        .replace_all(content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    let normalize_re = Regex::new(r#""(?P<key>threads_per_segment|thread_count)"\s*:\s*[^"]*"#)
        .expect("invalid normalize regex");

    let content = normalize_re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    let input_toml_re =
        Regex::new(r#""input_toml"\s*:\s*"(?:[^"\\]|\\.)*""#).expect("invalid input_toml regex");

    let content = input_toml_re
        .replace_all(&content, r#""input_toml": "_IGNORED_""#)
        .into_owned();

    if let Some(input_dir) = input_dir {
        content.replace(&format!("{}/", input_dir.to_string_lossy()), "")
    } else {
        content
    }
}

/// # Panics
/// When the internal regexps get broken
#[must_use]
pub fn normalize_progress_content(content: &str) -> String {
    let float_re = Regex::new(r"\d+[._0-9]*").expect("invalid float regex");
    let normalized = float_re.replace_all(content, "_IGNORED_").into_owned();

    let int_re = Regex::new(r"\b\d+\b").expect("invalid int regex");
    let normalized = int_re.replace_all(&normalized, "_IGNORED_").into_owned();
    //it's not quite deterministic with the last (few) Processed in output order.
    let normalized = normalized.replace("Final block passed Progress stage.\n", "");

    // Strip absolute paths, preserving any separator character that precedes them.
    // e.g. "from /tmp/abc/foo.fq" -> "from foo.fq" (space preserved).
    let file_re =
        Regex::new("(?:^|(?P<sep>[^A-Za-z0-9._-]))(/(?:[^/\\s]+/)*(?P<filename>[^/\\s]+))")
            .expect("invalid file regex");
    file_re
        .replace_all(&normalized, "${sep}${filename}")
        .into_owned()
}

fn find_output_files(dir: &Path, prefix: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let full = dir.join(prefix);
    let dir = full
        .parent()
        .expect("Must have a parent after dir joining, no?");
    let prefix = full
        .file_name()
        .expect("Must have a file name after joining dir and prefix")
        .to_string_lossy()
        .to_string();

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && file_name.starts_with(&prefix)
        {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn is_compressed_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext, "gz" | "gzip" | "zst" | "zstd")
    } else {
        false
    }
}

pub fn decompress_file(path: &Path) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open compressed file: {}", path.display()))?;

    let (mut reader, _format) = niffler::send::get_reader(Box::new(file)).with_context(|| {
        // cov:excl-start
        format!(
            "Failed to create decompression reader for: {}",
            path.display()
        )
    })?;
    // cov:excl-stop

    let mut decompressed = Vec::new();
    reader
        .read_to_end(&mut decompressed)
        .with_context(|| format!("Failed to decompress file: {}", path.display()))?;

    Ok(decompressed)
}

#[must_use]
#[expect(clippy::cast_precision_loss, reason = "loss is acceptable")]
pub fn calculate_size_difference_percent(len_a: u64, len_b: u64) -> f64 {
    if len_a > 0 {
        ((len_b as f64 - len_a as f64).abs() / len_a as f64) * 100.0
    } else if len_b > 0 {
        100.0
    } else {
        0.0
    }
}

enum ExpectedFailure {
    ExactText(String),
    Regex(Regex),
}

// cov:excl-start
impl std::fmt::Display for ExpectedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedFailure::ExactText(text) => write!(f, "{text}"),
            ExpectedFailure::Regex(regex) => write!(f, "/{}/", regex.as_str()),
        }
    }
}
// cov:excl-stop

impl ExpectedFailure {
    fn new(toml_dir: &Path, key: &str) -> Result<Option<Self>> {
        let expected_failure_file = toml_dir.join(format!("expected_{key}.txt"));
        let expected_failure_regex_file = toml_dir.join(format!("expected_{key}.regex"));

        if expected_failure_file.exists() && expected_failure_regex_file.exists() {
            // cov:excl-start
            bail!(
                "Both expected_failure.txt and expected_failure.regex files exist in {}. Please provide only one.",
                toml_dir.display()
            );
            // cov:excl-stop
        }

        if expected_failure_file.exists() {
            let content = ex::fs::read_to_string(&expected_failure_file)
                .context("Read expected failure file")?
                .trim()
                .to_string();
            // cov:excl-start
            assert!(
                content.trim() != "",
                "{}.txt was empty!",
                expected_failure_file.display()
            );
            // cov:excl-stop
            Ok(Some(ExpectedFailure::ExactText(content)))
        } else if expected_failure_regex_file.exists() {
            let content = ex::fs::read_to_string(&expected_failure_regex_file)
                .context("Read expected failure regex file")?
                .trim()
                .to_string();
            // cov:excl-start
            assert!(
                content.trim() != "",
                "{}.txt was empty!",
                expected_failure_regex_file.display()
            );
            // cov:excl-stop
            let regex = Regex::new(&content).context("Compile expected failure regex failed")?;
            Ok(Some(ExpectedFailure::Regex(regex)))
        } else {
            Ok(None)
        }
    }

    fn validate_expected_failure(&self, stderr: &str, temp_toml_path: &Path) -> Result<()> {
        let stderr = if std::env::var("RUST_BACKTRACE").is_ok() {
            // cov:excl-start
            strip_backtrace(stderr)
            // cov:excl-stop
        } else {
            Cow::Borrowed(stderr)
        };
        //replace url and version from error help
        let doc_url = format!(
            "{}v{}/docs/redirects/",
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_VERSION")
        );
        let stderr = stderr.replace(
            &doc_url,
            "https://doc_url.example/version-stripped-from-test/docs/redirects/",
        );

        //write to stderr file
        std::fs::write(
            temp_toml_path
                .parent()
                .expect("No parent for temp_toml_path?")
                .join("stderr"),
            &stderr,
        )
        .context("Failed to write actual stderr to file")?;

        match self {
            ExpectedFailure::ExactText(expected_text) => {
                if !stderr.contains(expected_text) {
                    bail!(
                        "fastqrab did not fail in the way that was expected.\nExpected message (substring): {expected_text}\nActual stderr: \n{stderr}"
                    );
                }
                if stderr.matches(expected_text).count() > 1 {
                    bail!(
                        "fastqrab failed in the expected way, but the expected message was found multiple times ({}). This may indicate an unexpected duplication of error messages.\nExpected message (substring): {}\nActual stderr: \n{}",
                        stderr.matches(expected_text).count(),
                        expected_text,
                        stderr
                    );
                }
            }
            ExpectedFailure::Regex(expected_regex) => {
                if !expected_regex.is_match(&stderr) {
                    bail!(
                        "fastqrab did not fail in the way that was expected.\nExpected message (regex): {}\nActual stderr: {}",
                        expected_regex.as_str(),
                        stderr
                    );
                }
            }
        }
        Ok(())
    }
}

fn strip_backtrace(stderr: &str) -> Cow<'_, str> {
    let mut out = Vec::new();
    let lines = stderr.split('\n');
    let mut outside = true;
    for line in lines {
        if outside {
            if line.trim().eq_ignore_ascii_case("stack backtrace:") {
                outside = false;
            } else {
                out.push(line);
            }
        } else if line.trim().is_empty() {
            outside = true;
        }
    }
    Cow::Owned(out.join("\n"))
}

const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

fn run_command_with_timeout(cmd: &mut std::process::Command) -> Result<std::process::Output> {
    use std::io::Read;
    use wait_timeout::ChildExt;

    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn command")?;

    if let Some(status) = child.wait_timeout(COMMAND_TIMEOUT)? {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut reader) = child.stdout.take() {
            reader.read_to_end(&mut stdout)?;
        } // cov:excl-line
        if let Some(mut reader) = child.stderr.take() {
            reader.read_to_end(&mut stderr)?;
        } // cov:excl-line
        Ok(std::process::Output {
            status,
            stdout,
            stderr,
        })
    } else {
        // not in coverage, this is a last resort to force tests
        // to come back.
        // cov:excl-start
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
        // cov:excl-stop
    }
}

fn create_symlinks_for_files(
    value: &toml::Value,
    source_dir: &Path,
    target_dir: &Path,
) -> Result<()> {
    if let Some(path_str) = value.as_str() {
        if path_str != STDIN_MAGIC_PATH {
            let source_path = source_dir.join(path_str);
            let target_path = target_dir.join(path_str);

            // Create parent directories if they don't exist
            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    // cov:excl-start
                    format!(
                        "Failed to create parent directories for {}",
                        target_path.display()
                    )
                })?;
                // cov:excl-stop
            } // cov:excl-line

            create_symlink(&source_path, &target_path)?;
        }
    } else if let Some(paths) = value.as_array() {
        for v in paths {
            if let Some(path_str) = v.as_str()
                && path_str != STDIN_MAGIC_PATH
            {
                let source_path = source_dir.join(path_str);
                let target_path = target_dir.join(path_str);

                // Create parent directories if they don't exist
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        // cov:excl-start
                        format!(
                            "Failed to create parent directories for {}",
                            target_path.display()
                        )
                    })?;
                    // cov:excl-stop
                } // cov:excl-line

                create_symlink(&source_path, &target_path)?;
            }
            // else: non-string value (e.g. integer) — skip silently; the
            // processor will report the type error during config validation.
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    // Use symlink_metadata (does NOT follow symlinks) so dangling symlinks are
    // detected as already existing and we don't attempt to recreate them.
    if std::fs::symlink_metadata(target).is_err() {
        std::os::unix::fs::symlink(source, target).with_context(|| {
            // cov:excl-start
            format!(
                "Failed to create symlink from {} to {}",
                source.display(),
                target.display()
            )
        })?;
        // cov:excl-stop
    }
    Ok(())
}

#[mutants::skip]
#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> Result<()> {
    if std::fs::symlink_metadata(target).is_err() {
        if source.is_dir() {
            std::os::windows::fs::symlink_dir(source, target).with_context(|| {
                format!(
                    "Failed to create directory symlink from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
        } else {
            std::os::windows::fs::symlink_file(source, target).with_context(|| {
                format!(
                    "Failed to create file symlink from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_calculate_size_difference_percent() {
        use super::calculate_size_difference_percent;

        let test_cases = vec![
            (100, 105, 5.0),
            (100, 95, 5.0),
            (100, 97, 3.0),
            (0, 100, 100.0),
            (100, 0, 100.0),
            (0, 0, 0.0),
            (200, 210, 5.0),
            (200, 190, 5.0),
        ];

        for (len_a, len_b, expected) in test_cases {
            let result = calculate_size_difference_percent(len_a, len_b);
            assert!(
                (result - expected).abs() < f64::EPSILON,
                "Failed for len_a: {len_a}, len_b: {len_b}: expected {expected}, got {result}",
            );
        }
    }

    #[test]
    fn test_decompress_file() {
        use super::decompress_file;
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut temp_file, flate2::Compression::default());
            encoder
                .write_all(b"Hello, world!")
                .expect("Failed to write to encoder");
            encoder.finish().expect("Failed to finish encoding");
        }

        let decompressed_data =
            decompress_file(temp_file.path()).expect("Failed to decompress file");

        assert_eq!(decompressed_data, b"Hello, world!");
    }

    #[test]
    fn test_strip_backtrace() {
        use super::strip_backtrace;

        let stderr = "running 3 tests
test pipeline::tests::test_checked_f64_to_u16 ... ok
test cli::verify::test::test_calculate_size_difference_percent ... ok
test cli::verify::test::test_decompress_file ... FAILED

failures:

---- cli::verify::test::test_decompress_file stdout ----

thread 'cli::verify::test::test_decompress_file' (1426326) panicked at fastqrab/src/cli/verify.rs:1008:9:
explicit panic
stack backtrace:
   0: __rustc::rust_begin_unwind
             at /rustc/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/library/std/src/panicking.rs:698:5
   1: core::panicking::panic_fmt
             at /rustc/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/library/core/src/panicking.rs:80:14
   2: core::panicking::panic
             at /rustc/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/library/core/src/panicking.rs:150:5
   3: fastqrab::cli::verify::test::test_decompress_file
             at ./src/cli/verify.rs:1008:9
   4: fastqrab::cli::verify::test::test_decompress_file::{{closure}}
             at ./src/cli/verify.rs:989:30
   5: core::ops::function::FnOnce::call_once
             at /rustc/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/library/core/src/ops/function.rs:250:5
   6: core::ops::function::FnOnce::call_once
             at /rustc/ded5c06cf21d2b93bffd5d884aa6e96934ee4234/library/core/src/ops/function.rs:250:5
note: Some details are omitted, run with `RUST_BACKTRACE=full` for a verbose backtrace.


failures:
    cli::verify::test::test_decompress_file";
        let should = "running 3 tests
test pipeline::tests::test_checked_f64_to_u16 ... ok
test cli::verify::test::test_calculate_size_difference_percent ... ok
test cli::verify::test::test_decompress_file ... FAILED

failures:

---- cli::verify::test::test_decompress_file stdout ----

thread 'cli::verify::test::test_decompress_file' (1426326) panicked at fastqrab/src/cli/verify.rs:1008:9:
explicit panic

failures:
    cli::verify::test::test_decompress_file";
        println!("{}", strip_backtrace(stderr));
        //dump both to a file for diff
        std::fs::write("actual_stderr.txt", strip_backtrace(stderr).to_string())
            .expect("Failed to write actual stderr to file");
        std::fs::write("expected_stderr.txt", should)
            .expect("Failed to write expected stderr to file");
        assert_eq!(strip_backtrace(stderr), should);
    }
}
