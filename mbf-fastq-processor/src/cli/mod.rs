use anyhow::{bail, Context, Result};
use regex::Regex;
use std::borrow::Cow;
use std::path::Path;
use std::time::Duration;

pub mod process;
pub mod validate;
pub mod verify;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(60);

pub(crate) fn improve_error_messages(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
    let mut e = extend_with_step_annotation(e, raw_toml);
    let msg = format!("{:?}", e);
    let barcode_regexp = Regex::new("barcodes.[^:]+: invalid type: sequence,")
        .expect("hardcoded regex pattern is valid");
    if barcode_regexp.is_match(&msg) {
        e = e.context("Use `[barcode.<name>]` instead of `[[barcode.<name>]]` in your config");
    }
    let options_search = "options[0]: invalid type: map, expected usize";
    if msg.contains(options_search) {
        e = e.context(
            "The 'options' field should be a table, not an array. Use [options], not [[options]]",
        );
    }
    let mistyped_input = "invalid type: sequence, expected struct __ImplEDeserializeForInput";
    if msg.contains(mistyped_input) {
        e = e.context("(input): The 'input' section should be a table, not an array. Use [input] instead of [[input]]");
    } else {
        let mistyped_input = "expected struct __ImplEDeserializeForInput";
        if msg.contains(mistyped_input) {
            e = e.context("(input): The 'input' section should be a table of segment = [filenames,...]. Example:\n[input]\nread1 = 'filename.fq'");
        }
    }
    let nested_input = "input: invalid type: map, expected string or list of strings";
    if msg.contains(nested_input) {
        e = e.context("x.y as key in TOML means 'a map below the current [section]. You are probably trying for a segment name with a dot (not allowed, remove dot), or tried [input] output.prefix, but you need [output]");
    }
    e
}

fn extend_with_step_annotation(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
    let msg = format!("{:?}", e);
    let step_regex = Regex::new(r"step.(\d+).").expect("hardcoded regex pattern is valid");
    if let Some(matches) = step_regex.captures(&msg) {
        let step_no = &matches[1];
        let step_int = step_no.parse::<usize>().unwrap_or(0);
        let parsed = toml::from_str::<toml::Value>(raw_toml);
        if let Ok(parsed) = parsed
            && let Some(step) = parsed.get("step")
            && let Some(steps) = step.as_array()
            && let Some(step_no_i) = steps.get(step_int)
            && let Some(action) = step_no_i.get("action").and_then(|v| v.as_str())
        {
            return e.context(format!(
                "Error in Step {step_no} (0-based), action = {action}"
            ));
        }
    }
    e
}

fn strip_backtrace<'a>(stderr: &'a str) -> Cow<'a, str> {
    let mut out = Vec::new();
    let lines = stderr.split('\n');
    let mut outside = true;
    for line in lines {
        if outside {
            if line.trim() == "Stack backtrace:" {
                outside = false;
            } else {
                out.push(line)
            }
        } else {
            if line.trim().is_empty() {
                outside = true;
            }
        }
    }
    Cow::Owned(out.join("\n"))
}

pub fn join_nonempty<'a>(
    parts: impl IntoIterator<Item = &'a str>,
    separator: &str,
) -> String {
    let mut iter = parts.into_iter().filter(|part| !part.is_empty());
    let mut result = String::new();
    if let Some(first) = iter.next() {
        result.push_str(first);
        for part in iter {
            result.push_str(separator);
            result.push_str(part);
        }
    }
    result
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

pub fn calculate_size_difference_percent(len_a: u64, len_b: u64) -> f64 {
    if len_a > 0 {
        ((len_b as f64 - len_a as f64).abs() / len_a as f64) * 100.0
    } else if len_b > 0 {
        100.0
    } else {
        0.0
    }
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
                    crate::normalize_progress_content(&expected_str),
                    crate::normalize_progress_content(&actual_str),
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
                    crate::normalize_report_content(&expected_str, None),
                    crate::normalize_report_content(&actual_str, Some(input_dir)),
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

pub(crate) fn find_output_files(dir: &Path, prefix: &str) -> Result<Vec<std::path::PathBuf>> {
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

pub(crate) fn run_command_with_timeout(cmd: &mut std::process::Command) -> Result<std::process::Output> {
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

pub(crate) enum ExpectedFailure {
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

    fn validate_expected_failure(&self, stderr: &str) -> Result<()> {
        let stderr = if std::env::var("RUST_BACKTRACE").is_ok() {
            strip_backtrace(stderr)
        } else {
            Cow::Borrowed(stderr)
        };

        match self {
            ExpectedFailure::ExactText(expected_text) => {
                if !stderr.contains(expected_text) {
                    bail!(
                        "mbf-fastq-processor did not fail in the way that was expected.\nExpected message (substring): {}\nActual stderr: \n{}",
                        expected_text,
                        stderr
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
            ExpectedFailure::ExactText(text) => write!(f, "ExactText({})", text),
            ExpectedFailure::Regex(regex) => write!(f, "Regex({})", regex.as_str()),
        }
    }
}

pub(crate) fn make_toml_path_absolute(value: &mut toml::Value, toml_dir: &Path) {
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

pub(crate) fn copy_input_file(value: &mut toml::Value, source_dir: &Path, target_dir: &Path) -> Result<()> {
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
