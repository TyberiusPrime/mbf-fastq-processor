#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]
#![allow(clippy::default_trait_access)] //when I say default::Default, that's future proofing for type changes...

use anyhow::{Context, Result, bail};
use config::Config;
use output::OutputRunMarker;
use regex::Regex;
use std::io::Write;
use std::path::Path;
use transformations::Transformation;

pub mod config;
pub mod cookbooks;
pub mod demultiplex;
mod dna;
pub mod documentation;
pub mod interactive;
pub mod io;
pub mod list_steps;
mod output;
mod pipeline;
mod pipeline_workpool;
mod transformations;

pub use io::FastQRead;

#[allow(clippy::similar_names)] // I like rx/tx nomenclature
#[allow(clippy::too_many_lines)] //todo: this is true.
pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let start_time = std::time::Instant::now();
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;
    parsed.check()?;
    let (mut parsed, report_labels) = Transformation::expand(parsed);
    let marker_prefix = parsed
        .output
        .as_ref()
        .expect("config.check() ensures output is present")
        .prefix
        .clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.preexisting();
    //parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
    let is_benchmark = parsed
        .benchmark
        .as_ref()
        .is_some_and(|b| b.enable && !b.quiet);
    #[allow(clippy::if_not_else)]
    {
        let run = pipeline::RunStage0::new(&parsed);
        let run = run.configure_demultiplex_and_init_stages(
            &mut parsed,
            &output_directory,
            allow_overwrite,
        )?;
        let run = run.create_input_threads(&parsed)?;
        let run = run.create_stage_threads(&mut parsed);
        let parsed = parsed; //after this, stages are transformed and ready, and config is read only.
        let run = run.create_output_threads(&parsed, report_labels, raw_config)?;
        let run = run.join_threads();
        //
        //promote all panics to actual process failures with exit code != 0
        let errors = run.errors;

        if !errors.is_empty() {
            bail!(errors.join("\n"));
        }
        //assert!(errors.is_empty(), "Error in threads occured: {errors:?}");

        //ok all this needs is a buffer that makes sure we reorder correctly at the end.
        //and then something block based, not single reads to pass between the threads.
        drop(parsed);
    }
    if is_benchmark {
        let elapsed = start_time.elapsed();
        println!(
            "Benchmark completed in {:.2?} seconds",
            elapsed.as_secs_f64()
        );
    }

    marker.mark_complete()?;
    Ok(())
}

/// Validates a configuration file without requiring input files to exist
/// Returns Ok(warnings) if validation succeeds, Err if there are actual errors
pub fn validate_config(toml_file: &Path) -> Result<Vec<String>> {
    use std::fs;

    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;

    // Run validation with validation mode enabled (this initializes structured input)
    parsed.check_for_validation()?;

    // Get the directory containing the TOML file to resolve relative paths
    let toml_dir = toml_file.parent().unwrap_or_else(|| Path::new("."));

    // Collect warnings about missing files after validation
    let mut warnings = Vec::new();

    // Check if input files exist and collect warnings
    match &parsed.input.structured {
        Some(config::StructuredInput::Interleaved { files, .. }) => {
            for file in files {
                if file != config::STDIN_MAGIC_PATH {
                    let file_path = toml_dir.join(file);
                    if fs::metadata(&file_path).is_err() {
                        warnings.push(format!("Input file not found: {file}"));
                    }
                }
            }
        }
        Some(config::StructuredInput::Segmented { segment_files, .. }) => {
            for (segment_name, files) in segment_files {
                for file in files {
                    if file != config::STDIN_MAGIC_PATH {
                        let file_path = toml_dir.join(file);
                        if fs::metadata(&file_path).is_err() {
                            warnings.push(format!(
                                "Input file not found in segment '{segment_name}': {file}"
                            ));
                        }
                    }
                }
            }
        }
        None => {}
    }

    Ok(warnings)
}

fn make_toml_path_absolute(value: &mut toml::Value, toml_dir: &Path) {
    if let Some(path_str) = value.as_str() {
        if path_str != config::STDIN_MAGIC_PATH {
            let abs_path = toml_dir.join(path_str);
            *value = toml::Value::String(abs_path.to_string_lossy().to_string());
        }
    } else if let Some(paths) = value.as_array() {
        let new_paths: Vec<toml::Value> = paths
            .iter()
            .map(|v| {
                if let Some(path_str) = v.as_str() {
                    if path_str == config::STDIN_MAGIC_PATH {
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

/// Verifies that running the configuration produces outputs matching expected outputs
/// in the directory where the TOML file is located
#[allow(clippy::too_many_lines)]
pub fn verify_outputs(toml_file: &Path, output_dir: Option<&Path>) -> Result<()> {
    // Get the directory containing the TOML file
    let toml_file_abs = toml_file.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize TOML file path: {}",
            toml_file.display()
        )
    })?;
    let toml_dir = toml_file_abs.parent().unwrap_or_else(|| Path::new("."));
    let toml_dir = toml_dir.to_path_buf();

    // Read the original TOML content
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    // Parse the TOML to extract configuration
    let parsed = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;

    if let Some(benchmark) = &parsed.benchmark
        && benchmark.enable
    {
        bail!(
        "This is a benchmarking configuration, can't be verified. Turn off benchmark.enable in your toml?"
    )}

    // Get the output configuration
    let output_config = parsed
        .output
        .as_ref()
        .context("No output section found in configuration")?;
    let output_prefix = output_config.prefix.clone();
    let uses_stdout = output_config.stdout;

    // Create a temporary directory for running the test
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    // Parse the TOML as a generic toml::Value so we can modify it
    let mut toml_value: toml::Value =
        toml::from_str(&raw_config).context("Failed to parse TOML for modification")?;

    // Convert input file paths to absolute paths
    if let Some(input_table) = toml_value.get_mut("input").and_then(|v| v.as_table_mut()) {
        // Handle different input file fields
        let field_names: Vec<String> = input_table.keys().cloned().collect();
        for field_name in &field_names {
            if field_name == "interleaved" || field_name == "options" {
                continue; // handled separately
            }
            if let Some(value) = input_table.get_mut(field_name) {
                make_toml_path_absolute(value, &toml_dir);
            }
        }
    }

    // Convert file paths in step sections to absolute paths
    if let Some(steps) = toml_value.get_mut("step").and_then(|v| v.as_array_mut()) {
        for step in steps {
            if let Some(step_table) = step.as_table_mut() {
                // Handle 'filename' field in steps (used by TagOtherFileByName, etc.)
                for filename_key in ["filename", "filenames", "files"] {
                    if let Some(value) = step_table.get_mut(filename_key) {
                        make_toml_path_absolute(value, &toml_dir);
                    }
                }
                // // Add other file path fields as needed
                // for field_name in ["input_file", "file", "path"] {
                //     if let Some(field_value) = step_table.get_mut(field_name) {
                //         if let Some(path_str) = field_value.as_str() {
                //             if path_str != config::STDIN_MAGIC_PATH {
                //                 let abs_path = toml_dir.join(path_str);
                //                 *field_value =
                //                     toml::Value::String(abs_path.to_string_lossy().to_string());
                //             }
                //         }
                //     }
                // }
            }
        }
    }

    // Write the modified TOML to the temp directory
    let temp_toml_path = temp_path.join("config.toml");
    let modified_toml =
        toml::to_string_pretty(&toml_value).context("Failed to serialize modified TOML")?;
    ex::fs::write(&temp_toml_path, modified_toml)
        .context("Failed to write modified TOML to temp directory")?;

    // Run processing in the temp directory
    // capture stdout & stderr - claude, this means we must run ourselves as an external command!
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Check if configuration uses stdin and if we have a stdin file
    let uses_stdin = raw_config.contains(config::STDIN_MAGIC_PATH);
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

    let mut command = std::process::Command::new(current_exe);
    command
        .arg("process")
        .arg(&temp_toml_path)
        //.arg("--allow-overwrite")
        .current_dir(temp_path);

    let output = if let Some(stdin_path) = stdin_file {
        // Pipe stdin from file
        let stdin_content = ex::fs::read(&stdin_path)
            .with_context(|| format!("Failed to read stdin file: {}", stdin_path.display()))?;

        let mut child = command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn mbf-fastq-processor subprocess")?;

        // Get stdin handle and write content
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
        // No stdin needed
        command
            .output()
            .context("Failed to execute mbf-fastq-processor subprocess")?
    };
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        bail!(
            "Processing failed with exit code {:?}. stderr: {}",
            output.status.code(),
            stderr
        );
    }

    // Write stdout and stderr to files for comparison
    if !output.stdout.is_empty() {
        ex::fs::write(temp_path.join("stdout"), &output.stdout)
            .context("Failed to write stdout to temp directory")?;
    }
    if !output.stderr.is_empty() {
        ex::fs::write(temp_path.join("stderr"), &output.stderr)
            .context("Failed to write stderr to temp directory")?;
    }

    // Compare outputs
    let expected_dir = &toml_dir;
    let actual_dir = temp_path;

    // Compare each output file (skip if output goes to stdout)
    let mut mismatches = Vec::new();
    if !uses_stdout {
        // Find all output files in the expected directory with the given prefix
        let expected_files = find_output_files(expected_dir, &output_prefix)?;

        if expected_files.is_empty() {
            bail!(
                "No expected output files found in {} with prefix '{}'",
                expected_dir.display(),
                output_prefix
            );
        }

        for expected_file in &expected_files {
            let file_name = expected_file
                .file_name()
                .context("Failed to get file name")?;
            let actual_file = actual_dir.join(file_name);

            if !actual_file.exists() {
                mismatches.push(format!(
                    "Missing output file: {}",
                    file_name.to_string_lossy()
                ));
                continue;
            }

            // Compare file contents
            if let Err(e) = compare_files(expected_file, &actual_file) {
                mismatches.push(format!("{}: {}", file_name.to_string_lossy(), e));
            }
        }
    }

    // Compare stdout and stderr files if they exist
    for stream_name in ["stdout", "stderr"] {
        let expected_stream_file = expected_dir.join(stream_name);
        let actual_stream_file = actual_dir.join(stream_name);

        if expected_stream_file.exists() {
            if !actual_stream_file.exists() {
                mismatches.push(format!("Missing {stream_name} file"));
            } else if let Err(e) = compare_files(&expected_stream_file, &actual_stream_file) {
                mismatches.push(format!("{stream_name}: {e}"));
            }
        } else if actual_stream_file.exists() {
            mismatches.push(format!("Unexpected {stream_name} file"));
        }
    }

    // Check for extra files in actual output (skip if output goes to stdout)
    if !uses_stdout {
        let actual_files = find_output_files(actual_dir, &output_prefix)?;
        for actual_file in &actual_files {
            let file_name = actual_file.file_name().context("Failed to get file name")?;
            let expected_file = expected_dir.join(file_name);

            //claude: not for stdout/stderr.
            if !expected_file.exists() {
                mismatches.push(format!(
                    "Unexpected output file: {}",
                    file_name.to_string_lossy()
                ));
            }
        }
    }

    if !mismatches.is_empty() {
        // If output_dir is provided, copy tempdir contents there with normalizers applied
        if let Some(output_dir) = output_dir {
            // Remove output_dir if it exists
            if output_dir.exists() {
                ex::fs::remove_dir_all(output_dir).with_context(|| {
                    format!(
                        "Failed to remove existing output directory: {}",
                        output_dir.display()
                    )
                })?;
            }

            // Create output_dir
            ex::fs::create_dir_all(output_dir).with_context(|| {
                format!(
                    "Failed to create output directory: {}",
                    output_dir.display()
                )
            })?;

            // Copy all files from tempdir to output_dir with normalizers applied
            for entry in ex::fs::read_dir(actual_dir).with_context(|| {
                format!("Failed to read temp directory: {}", actual_dir.display())
            })? {
                let entry = entry?;
                let src_path = entry.path();
                if src_path.is_file() {
                    let file_name = src_path.file_name().context("Failed to get file name")?;
                    let dest_path = output_dir.join(file_name);

                    // Check if this is a file that needs normalization
                    if src_path
                        .extension()
                        .is_some_and(|ext| ext == "json" || ext == "html" || ext == "progress")
                    {
                        let content = ex::fs::read_to_string(&src_path).with_context(|| {
                            format!("Failed to read file: {}", src_path.display())
                        })?;

                        let normalized =
                            if src_path.extension().is_some_and(|ext| ext == "progress") {
                                normalize_progress_content(&content)
                            } else {
                                normalize_report_content(&content)
                            };

                        ex::fs::write(&dest_path, normalized).with_context(|| {
                            format!("Failed to write normalized file: {}", dest_path.display())
                        })?;
                    } else {
                        // Copy file as-is
                        ex::fs::copy(&src_path, &dest_path).with_context(|| {
                            format!(
                                "Failed to copy file from {} to {}",
                                src_path.display(),
                                dest_path.display()
                            )
                        })?;
                    }
                }
            }
        }

        bail!("Output verification failed:\n  {}", mismatches.join("\n  "));
    }

    Ok(())
}

/// Find all output files in a directory with a given prefix
fn find_output_files(dir: &Path, prefix: &str) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && let Some(file_name) = path.file_name().and_then(|n| n.to_str())
            && file_name.starts_with(prefix)
        {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

/// Normalize JSON/HTML report content for comparison
/// Replaces dynamic fields (paths, versions, etc.) with fixed placeholders
#[must_use]
pub fn normalize_report_content(content: &str) -> String {
    // Normalize version, working_directory, cwd, repository fields
    let normalize_re = Regex::new(
        r#""(?P<key>version|program_version|cwd|working_directory|repository)"\s*:\s*"[^"]*""#,
    )
    .expect("invalid normalize regex");

    let normalized = normalize_re
        .replace_all(content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    // Normalize input_toml field (contains full TOML with absolute paths)
    let input_toml_re =
        Regex::new(r#""input_toml"\s*:\s*"(?:[^"\\]|\\.)*""#).expect("invalid input_toml regex");

    let normalized = input_toml_re
        .replace_all(&normalized, r#""input_toml": "_IGNORED_""#)
        .into_owned();

    // Normalize file paths - convert absolute paths to basenames
    // This handles paths in input_files section
    let path_re = Regex::new(r#""(/[^"]+)""#).expect("invalid path regex");

    path_re
        .replace_all(&normalized, |caps: &regex::Captures| {
            let path = &caps[1];
            let basename = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path);
            format!(r#""{basename}""#)
        })
        .into_owned()
}

#[must_use]
pub fn normalize_progress_content(content: &str) -> String {
    // Normalize timing values, rates, and elapsed time in .progress files
    let float_re = Regex::new(r"\d+[._0-9]*").expect("invalid float regex");
    let normalized = float_re.replace_all(content, "_IGNORED_").into_owned();

    // Also normalize pure integers that represent time/counts that might vary
    let int_re = Regex::new(r"\b\d+\b").expect("invalid int regex");
    let normalized = int_re.replace_all(&normalized, "_IGNORED_").into_owned();

    //also normalize file paths to just the name
    let file_re = Regex::new("(?:^|[^A-Za-z0-9._-])(/(?:[^/\\s]+/)*([^/\\s]+))").expect("invalid file regex");
    file_re.replace_all(&normalized,  "$2").into_owned()
}

/// Check if a file is compressed based on its extension
fn is_compressed_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext, "gz" | "gzip" | "zst" | "zstd")
    } else {
        false
    }
}

/// Decompress a file and return its uncompressed content
fn decompress_file(path: &Path) -> Result<Vec<u8>> {
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

/// Compare two files byte-by-byte
#[allow(clippy::cast_precision_loss)]
fn compare_files(expected: &Path, actual: &Path) -> Result<()> {
    // Check if these are compressed files
    let is_compressed = is_compressed_file(expected) || is_compressed_file(actual);

    let (expected_bytes, actual_bytes) = if is_compressed {
        // For compressed files, compare uncompressed content
        let expected_uncompressed = decompress_file(expected)?;
        let actual_uncompressed = decompress_file(actual)?;

        // Check that compressed file sizes are within 5% of each other
        let expected_compressed_size = std::fs::metadata(expected)?.len();
        let actual_compressed_size = std::fs::metadata(actual)?.len();

        let size_diff_percent = if expected_compressed_size > 0 {
            ((actual_compressed_size as f64 - expected_compressed_size as f64).abs()
                / expected_compressed_size as f64)
                * 100.0
        } else if actual_compressed_size > 0 {
            100.0 // One is empty, one isn't
        } else {
            0.0 // Both are empty
        };

        if size_diff_percent > 5.0 {
            bail!(
                "Compressed file size difference too large: expected {expected_compressed_size} bytes, got {actual_compressed_size} bytes ({size_diff_percent}% difference)",
            );
        }

        (expected_uncompressed, actual_uncompressed)
    } else {
        // For uncompressed files, read directly
        let expected_bytes = std::fs::read(expected)
            .with_context(|| format!("Failed to read expected file: {}", expected.display()))?;
        let actual_bytes = std::fs::read(actual)
            .with_context(|| format!("Failed to read actual file: {}", actual.display()))?;
        (expected_bytes, actual_bytes)
    };

    // For JSON, HTML report files, and .progress files, normalize dynamic fields before comparison
    let (expected_normalized, actual_normalized) = if expected
        .extension()
        .is_some_and(|ext| ext == "json" || ext == "html" || ext == "progress")
    {
        let expected_str = String::from_utf8_lossy(&expected_bytes);
        let actual_str = String::from_utf8_lossy(&actual_bytes);

        let (expected_normalized, actual_normalized) =
            if expected.extension().is_some_and(|ext| ext == "progress") {
                // Handle .progress files
                (
                    normalize_progress_content(&expected_str),
                    normalize_progress_content(&actual_str),
                )
            } else {
                // Handle other JSON/HTML files
                (
                    normalize_report_content(&expected_str),
                    normalize_report_content(&actual_str),
                )
            };

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
        // Find first difference for better error message
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

pub(crate) fn join_nonempty<'a>(
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

fn improve_error_messages(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
    let msg = e.to_string();
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
