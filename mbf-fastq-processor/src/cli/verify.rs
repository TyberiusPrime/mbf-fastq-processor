use anyhow::{Context, Result, bail};
use ex::fs;
use regex::Regex;
use std::borrow::Cow;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

#[allow(clippy::too_many_lines)]
pub fn verify_outputs(
    toml_file: &Path,
    output_dir: Option<&Path>,
    unsafe_prep: bool,
) -> Result<()> {
    let toml_file_abs = toml_file.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize TOML file path: {}",
            toml_file.display()
        )
    })?;
    let toml_dir = toml_file_abs.parent().unwrap_or_else(|| Path::new("."));
    let toml_dir = toml_dir.to_path_buf();
    let output_dir = output_dir.map(|output_dir| {
        if output_dir.is_absolute() {
            output_dir.to_owned()
        } else {
            toml_dir.join(output_dir)
        }
    });

    let prep_script = toml_dir.join("prep.sh");
    let post_script = toml_dir.join("post.sh");
    let test_script = toml_dir.join("test.sh");

    let do_copy_input_files = toml_dir.join("copy_input").exists() || test_script.exists();

    let expected_validation_error = ExpectedFailure::new(&toml_dir, "error")?;
    let expected_validation_warning = ExpectedFailure::new(&toml_dir, "validation_warning")?;
    let expected_runtime_error = ExpectedFailure::new(&toml_dir, "runtime_error")?;

    let error_file_count =
        u8::from(expected_validation_error.is_some()) + u8::from(expected_runtime_error.is_some());
    if error_file_count > 1 {
        bail!(
            "Both expected_error(.txt|.regex) and expected_runtime_error(.txt|.regex) files exist. Please provide only one, depending on wether it's a validation or a processing error."
        );
    }

    let expected_failure = match (
        expected_validation_error.as_ref(),
        expected_runtime_error.as_ref(),
    ) {
        (Some(x), None) | (None, Some(x)) => Some(x),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!(),
    };

    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    let (output_prefix, uses_stdout) = {
        let result = crate::config::config_from_string(&raw_config);

        // let parsed = match result {
        //     Ok(config) => config,
        //     Err(e) => {
        //         return Err(anyhow::anyhow!("{}", e.pretty("config.toml")));
        //     }
        // };

        if let Ok(parsed) = &result
            && let Some(benchmark) = &parsed.benchmark
            && benchmark.enable
        {
            bail!(
                "This is a benchmarking configuration, which can't be verified for it's output (it has none). Maybe turn off benchmark.enable in your TOML, or use another configuration?"
            )
        }

        result
            .ok()
            .and_then(|parsed| parsed.output.as_ref().map(|o| (o.prefix.clone(), o.stdout)))
            .unwrap_or_else(|| ("missing_output_config".to_string(), false))
    };

    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = if let Some(output_dir) = output_dir.as_ref() {
        if output_dir.exists() {
            ex::fs::remove_dir_all(output_dir).with_context(|| {
                format!(
                    "Failed to remove existing output directory: {}",
                    output_dir.display()
                )
            })?;
        }
        std::fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                output_dir.display()
            )
        })?;
        output_dir
            .canonicalize()
            .expect("Failed to canonicalize output dir")
    } else {
        temp_dir.path().to_owned()
    };

    let mut toml_value: toml::Value =
        toml::from_str(&raw_config).context("Failed to parse TOML for modification")?;

    if do_copy_input_files {
        if let Some(input_table) = toml_value.get_mut("input").and_then(|v| v.as_table_mut()) {
            let field_names: Vec<String> = input_table.keys().cloned().collect();
            for field_name in &field_names {
                if field_name == "interleaved" || field_name == "options" {
                    continue;
                }
                if let Some(value) = input_table.get_mut(field_name) {
                    copy_input_file(value, &toml_dir, &temp_path)?;
                }
            }
        }
        if let Some(steps) = toml_value.get_mut("step").and_then(|v| v.as_array_mut()) {
            for step in steps {
                if let Some(step_table) = step.as_table_mut() {
                    for filename_key in ["filename", "filenames", "files"] {
                        if let Some(value) = step_table.get_mut(filename_key) {
                            copy_input_file(value, &toml_dir, &temp_path)?;
                        }
                    }
                }
            }
        }
        for entry in fs::read_dir(&toml_dir)? {
            let entry = entry?;
            let src_path = entry.path();
            if src_path.is_file()
                && let Some(file_name) = src_path.file_name()
            {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("input") && file_name_str != "input.toml" {
                    let dst_path = temp_path.join(file_name);
                    if !dst_path.exists() {
                        fs::copy(&src_path, &dst_path)?;
                    }
                }
            }
        }
    } else {
        if let Some(input_table) = toml_value.get_mut("input").and_then(|v| v.as_table_mut()) {
            let field_names: Vec<String> = input_table.keys().cloned().collect();
            for field_name in &field_names {
                if field_name == "interleaved" || field_name == "options" {
                    continue;
                }
                if let Some(value) = input_table.get_mut(field_name) {
                    make_toml_path_absolute(value, &toml_dir);
                }
            }
        }

        if let Some(steps) = toml_value.get_mut("step").and_then(|v| v.as_array_mut()) {
            for step in steps {
                if let Some(step_table) = step.as_table_mut() {
                    for filename_key in ["filename", "filenames", "files"] {
                        if let Some(value) = step_table.get_mut(filename_key) {
                            make_toml_path_absolute(value, &toml_dir);
                        }
                    }
                }
            }
        }
    }

    let temp_toml_path = temp_path.join("config.toml");
    let modified_toml =
        toml::to_string_pretty(&toml_value).context("Failed to serialize modified TOML")?;
    ex::fs::write(&temp_toml_path, modified_toml)
        .context("Failed to write modified TOML to temp directory")?;

    if unsafe_prep {
        if prep_script.exists() {
            #[cfg(not(target_os = "windows"))]
            let mut prep_command = {
                let mut command = std::process::Command::new("bash");
                command
                    .arg(prep_script.canonicalize().context("canonicalize prep.sh")?)
                    .current_dir(&temp_path);
                command
            };

            #[cfg(target_os = "windows")]
            let mut prep_command = {
                bail!("prep.sh execution on Windows is not currently supported");
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

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    let uses_stdin = raw_config.contains(crate::config::STDIN_MAGIC_PATH);
    let stdin_file = if uses_stdin {
        let stdin_path = toml_dir.join("stdin");
        if stdin_path.exists() {
            Some(stdin_path)
        } else {
            None
        }
    } else {
        None
    };

    if expected_validation_error.is_none() | expected_validation_warning.is_some() {
        let warnings =
            crate::cli::validate::validate_config(&temp_toml_path).with_context(|| {
                if expected_runtime_error.is_some() {
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
                    .validate_expected_failure(w, &temp_toml_path)
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

    if test_script.exists() {
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
    } else {
        let mut command = std::process::Command::new(current_exe);
        command
            .arg(if expected_validation_error.is_none() {
                "process"
            } else {
                "validate"
            })
            .arg("config.toml")
            .current_dir(&temp_path);

        let output = if let Some(stdin_path) = stdin_file {
            let stdin_content = ex::fs::read(&stdin_path)
                .with_context(|| format!("Failed to read stdin file: {}", stdin_path.display()))?;

            let mut child = command
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .context("Failed to spawn mbf-fastq-processor subprocess")?;

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(&stdin_content)
                    .context("Failed to write to subprocess stdin")?;
                stdin.flush().context("Failed to flush subprocess stdin")?;
                drop(stdin);
            }

            child
                .wait_with_output()
                .context("Failed to wait for subprocess completion")?
        } else {
            command
                .output()
                .context("Failed to execute mbf-fastq-processor subprocess")?
        };
        let stderr = String::from_utf8_lossy(&output.stderr);

        match (expected_failure.as_ref(), output.status.success()) {
            (Some(expected_failure_pattern), false) => {
                expected_failure_pattern.validate_expected_failure(&stderr, &temp_toml_path)?;
            }
            (Some(_), true) => {
                if expected_validation_error.is_some() {
                    bail!(
                        "Expected validation failure but 'validate' command succeeded. stderr: {stderr}"
                    );
                } else {
                    bail!(
                        "Expected runtime failure but 'process' command succeeded. stderr: {stderr}"
                    );
                };
            }
            (None, false) => {
                bail!(
                    "Processing failed with exit code {:?}. stderr: {}",
                    output.status.code(),
                    stderr
                );
            }
            (None, true) => {
                if !output.stdout.is_empty() {
                    ex::fs::write(temp_path.join("stdout"), &output.stdout)
                        .context("Failed to write stdout to temp directory")?;
                }
                if !output.stderr.is_empty() {
                    ex::fs::write(temp_path.join("stderr"), &output.stderr)
                        .context("Failed to write stderr to temp directory")?;
                }

                let mut mismatches = Vec::new();
                if post_script.exists() && unsafe_prep {
                    #[cfg(not(target_os = "windows"))]
                    let mut post_command = {
                        let mut command = std::process::Command::new("bash");
                        command
                            .arg(post_script.canonicalize().context("canonicalize post.sh")?)
                            .current_dir(&temp_path);
                        command
                    };

                    #[cfg(target_os = "windows")]
                    let mut post_command = {
                        bail!("post.sh execution on Windows is not currently supported");
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

                let expected_dir = &toml_dir;
                let actual_dir = temp_path;

                if !uses_stdout {
                    let expected_files =
                        find_output_files(expected_dir, &output_prefix).unwrap_or_default();

                    if expected_files.is_empty() {
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
                            mismatches.push(format!("Missing output file: {str_surplus}",));
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
                    let actual_files = find_output_files(&actual_dir, &output_prefix)?;
                    for actual_file in &actual_files {
                        let surplus = actual_file
                            .strip_prefix(&actual_dir)
                            .expect("Stripping the dir again should work...");
                        let str_surplus = surplus.to_string_lossy();
                        let expected_file = expected_dir.join(str_surplus.as_ref());
                        if !expected_file.exists() {
                            mismatches.push(format!("Unexpected output file: {str_surplus}",));
                        }
                    }
                }

                if !mismatches.is_empty() {
                    bail!("Output verification failed:\n  {}", mismatches.join("\n  "));
                }
            }
        }
    }

    if let Some(output_dir) = output_dir
        && output_dir.exists()
    {
        ex::fs::remove_dir_all(&output_dir).with_context(|| {
            format!(
                "Failed to remove existing output directory: {}",
                output_dir.display()
            )
        })?;
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
        println!("applying normalization to {}", expected.display());
        let expected_str = String::from_utf8_lossy(&expected_bytes);
        let actual_str = String::from_utf8_lossy(&actual_bytes);

        let (expected_normalized, actual_normalized) =
            if expected.extension().is_some_and(|ext| ext == "progress") {
                let res = (
                    normalize_progress_content(&expected_str),
                    normalize_progress_content(&actual_str),
                );
                std::fs::write(actual, &res.1).with_context(|| {
                    format!(
                        "Failed to write normalized actual report file: {}",
                        actual.display()
                    )
                })?;
                res
            } else {
                let res = (
                    normalize_report_content(&expected_str, None),
                    normalize_report_content(&actual_str, Some(input_dir)),
                );
                std::fs::write(actual, &res.1).with_context(|| {
                    format!(
                        "Failed to write normalized actual report file: {}",
                        actual.display()
                    )
                })?;
                res
            };

        if expected_normalized.is_empty() {
            bail!("expected file was empty after normalization - shouldn't be?");
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
        bail!("Content mismatch (no specific byte difference found)");
    }

    Ok(())
}

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

#[must_use]
pub fn normalize_progress_content(content: &str) -> String {
    let float_re = Regex::new(r"\d+[._0-9]*").expect("invalid float regex");
    let normalized = float_re.replace_all(content, "_IGNORED_").into_owned();

    let int_re = Regex::new(r"\b\d+\b").expect("invalid int regex");
    let normalized = int_re.replace_all(&normalized, "_IGNORED_").into_owned();

    let file_re =
        Regex::new("(?:^|[^A-Za-z0-9._-])(/(?:[^/\\s]+/)*([^/\\s]+))").expect("invalid file regex");
    file_re.replace_all(&normalized, "$2").into_owned()
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
        format!(
            "Failed to create decompression reader for: {}",
            path.display()
        )
    })?;

    let mut decompressed = Vec::new();
    reader
        .read_to_end(&mut decompressed)
        .with_context(|| format!("Failed to decompress file: {}", path.display()))?;

    Ok(decompressed)
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
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

impl ExpectedFailure {
    fn new(toml_dir: &Path, key: &str) -> Result<Option<Self>> {
        let expected_failure_file = toml_dir.join(format!("expected_{key}.txt"));
        let expected_failure_regex_file = toml_dir.join(format!("expected_{key}.regex"));

        if expected_failure_file.exists() && expected_failure_regex_file.exists() {
            bail!(
                "Both expected_failure.txt and expected_failure.regex files exist in {}. Please provide only one.",
                toml_dir.display()
            );
        }

        if expected_failure_file.exists() {
            let content = ex::fs::read_to_string(&expected_failure_file)
                .context("Read expected failure file")?
                .trim()
                .to_string();
            assert!(
                content.trim() != "",
                "{}.txt was empty!",
                expected_failure_file.display()
            );
            Ok(Some(ExpectedFailure::ExactText(content)))
        } else if expected_failure_regex_file.exists() {
            let content = ex::fs::read_to_string(&expected_failure_regex_file)
                .context("Read expected failure regex file")?
                .trim()
                .to_string();
            assert!(
                content.trim() != "",
                "{}.txt was empty!",
                expected_failure_regex_file.display()
            );
            let regex = Regex::new(&content).context("Compile expected failure regex failed")?;
            Ok(Some(ExpectedFailure::Regex(regex)))
        } else {
            Ok(None)
        }
    }

    fn validate_expected_failure(&self, stderr: &str, temp_toml_path: &Path) -> Result<()> {
        let stderr = if std::env::var("RUST_BACKTRACE").is_ok() {
            strip_backtrace(stderr)
        } else {
            Cow::Borrowed(stderr)
        };
        //replace url and version from error help
        let doc_url = format!(
            "{}v{}/docs/reference/",
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_VERSION")
        );
        let stderr = stderr.replace(
            &doc_url,
            "https://doc_url.example/version-stripped-from-test/docs/reference/",
        );

        //write to stderr file
        std::fs::write(temp_toml_path.parent().unwrap().join("stderr"), &stderr)
            .context("Failed to write actual stderr to file")?;

        match self {
            ExpectedFailure::ExactText(expected_text) => {
                if !stderr.contains(expected_text) {
                    bail!(
                        "mbf-fastq-processor did not fail in the way that was expected.\nExpected message (substring): {expected_text}\nActual stderr: \n{stderr}"
                    );
                }
                if stderr.matches(expected_text).count() > 1 {
                    bail!(
                        "mbf-fastq-processor failed in the expected way, but the expected message was found multiple times ({}). This may indicate an unexpected duplication of error messages.\nExpected message (substring): {}\nActual stderr: \n{}",
                        stderr.matches(expected_text).count(),
                        expected_text,
                        stderr
                    );
                }
            }
            ExpectedFailure::Regex(expected_regex) => {
                if !expected_regex.is_match(&stderr) {
                    bail!(
                        "mbf-fastq-processor did not fail in the way that was expected.\nExpected message (regex): {}\nActual stderr: {}",
                        expected_regex.as_str(),
                        stderr
                    );
                }
            }
        }
        Ok(())
    }
}

impl std::fmt::Display for ExpectedFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedFailure::ExactText(text) => write!(f, "ExactText({text})"),
            ExpectedFailure::Regex(regex) => write!(f, "Regex({})", regex.as_str()),
        }
    }
}

fn strip_backtrace(stderr: &str) -> Cow<'_, str> {
    let mut out = Vec::new();
    let lines = stderr.split('\n');
    let mut outside = true;
    for line in lines {
        if outside {
            if line.trim() == "Stack backtrace:" {
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

fn copy_input_file(value: &mut toml::Value, source_dir: &Path, target_dir: &Path) -> Result<()> {
    if let Some(path_str) = value.as_str() {
        if path_str != crate::config::STDIN_MAGIC_PATH {
            let out_path = target_dir.join(path_str);
            let input_path = source_dir.join(path_str);
            std::fs::copy(&input_path, &out_path).with_context(|| {
                format!(
                    "Failed to copy input file from {} to {}",
                    input_path.display(),
                    out_path.display(),
                )
            })?;
        }
        return Ok(());
    } else if let Some(paths) = value.as_array() {
        let new_paths: Result<Vec<()>> = paths
            .iter()
            .map(|v| {
                if let Some(path_str) = v.as_str() {
                    if path_str == crate::config::STDIN_MAGIC_PATH {
                        Ok(())
                    } else {
                        let out_path = target_dir.join(path_str);
                        let input_path = source_dir.join(path_str);
                        std::fs::copy(&input_path, &out_path).with_context(|| {
                            format!(
                                "Failed to copy input file from {} to {}",
                                input_path.display(),
                                out_path.display(),
                            )
                        })?;
                        Ok(())
                    }
                } else {
                    anyhow::bail!("Invalid toml value")
                }
            })
            .collect();
        new_paths?;
    }
    Ok(())
}

fn make_toml_path_absolute(value: &mut toml::Value, toml_dir: &Path) {
    if let Some(path_str) = value.as_str() {
        if path_str != crate::config::STDIN_MAGIC_PATH {
            let abs_path = toml_dir.join(path_str);
            *value = toml::Value::String(abs_path.to_string_lossy().to_string());
        }
    } else if let Some(paths) = value.as_array() {
        let new_paths: Vec<toml::Value> = paths
            .iter()
            .map(|v| {
                if let Some(path_str) = v.as_str() {
                    if path_str == crate::config::STDIN_MAGIC_PATH {
                        v.clone()
                    } else {
                        let abs_path = toml_dir.join(path_str);
                        toml::Value::String(abs_path.to_string_lossy().to_string())
                    }
                } else {
                    v.clone()
                }
            })
            .collect();
        *value = toml::Value::Array(new_paths);
    }
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
                "Failed for len_a: {}, len_b: {}: expected {}, got {}",
                len_a,
                len_b,
                expected,
                result
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
}
