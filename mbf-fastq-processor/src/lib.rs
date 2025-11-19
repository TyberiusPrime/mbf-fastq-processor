#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]
#![allow(clippy::default_trait_access)] //when I say default::Default, that's future proofing for type changes...

use anyhow::{Context, Result, bail};
use config::Config;
use output::OutputRunMarker;
use regex::Regex;
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
mod timing;
mod transformations;

pub use io::FastQRead;

#[allow(clippy::similar_names)] // I like rx/tx nomenclature
#[allow(clippy::too_many_lines)] //todo: this is true.
pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;
    parsed.check()?;
    let (mut parsed, report_labels) = Transformation::expand(parsed);
    let marker_prefix = parsed.output.as_ref().expect("config.check() ensures output is present").prefix.clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.preexisting();
    //parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
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

/// Verifies that running the configuration produces outputs matching expected outputs
/// in the directory where the TOML file is located
#[allow(clippy::too_many_lines)]
pub fn verify_outputs(toml_file: &Path) -> Result<()> {
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

    // Get the output prefix to know which files to compare
    let output_prefix = parsed
        .output
        .as_ref()
        .context("No output section found in configuration")?
        .prefix
        .clone();

    // Create a temporary directory for running the test
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    // Parse the TOML as a generic toml::Value so we can modify it
    let mut toml_value: toml::Value =
        toml::from_str(&raw_config).context("Failed to parse TOML for modification")?;

    // Convert input file paths to absolute paths
    if let Some(input_table) = toml_value.get_mut("input").and_then(|v| v.as_table_mut()) {
        // Handle different input file fields
        for field_name in &["read1", "read2", "index1", "index2", "interleaved"] {
            if let Some(value) = input_table.get_mut(*field_name) {
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
        }

        // Handle segments (more complex structure)
        if let Some(segments_table) = input_table
            .get_mut("segments")
            .and_then(|v| v.as_table_mut())
        {
            for (_segment_name, segment_value) in segments_table.iter_mut() {
                if let Some(segment_table) = segment_value.as_table_mut() {
                    for field_name in &["read1", "read2", "index1", "index2"] {
                        if let Some(value) = segment_table.get_mut(*field_name) {
                            if let Some(path_str) = value.as_str() {
                                if path_str != config::STDIN_MAGIC_PATH {
                                    let abs_path = toml_dir.join(path_str);
                                    *value =
                                        toml::Value::String(abs_path.to_string_lossy().to_string());
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
                                                toml::Value::String(
                                                    abs_path.to_string_lossy().to_string(),
                                                )
                                            }
                                        } else {
                                            v.clone()
                                        }
                                    })
                                    .collect();
                                *value = toml::Value::Array(new_paths);
                            }
                        }
                    }
                }
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
    run(&temp_toml_path, temp_path, true)
        .context("Failed to run processing in temporary directory")?;

    // Compare outputs
    let expected_dir = &toml_dir;
    let actual_dir = temp_path;

    // Find all output files in the expected directory with the given prefix
    let expected_files = find_output_files(expected_dir, &output_prefix)?;

    if expected_files.is_empty() {
        bail!(
            "No expected output files found in {} with prefix '{}'",
            expected_dir.display(),
            output_prefix
        );
    }

    // Compare each output file
    let mut mismatches = Vec::new();
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

    // Check for extra files in actual output
    let actual_files = find_output_files(actual_dir, &output_prefix)?;
    for actual_file in &actual_files {
        let file_name = actual_file.file_name().context("Failed to get file name")?;
        let expected_file = expected_dir.join(file_name);

        if !expected_file.exists() {
            mismatches.push(format!(
                "Unexpected output file: {}",
                file_name.to_string_lossy()
            ));
        }
    }

    if !mismatches.is_empty() {
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

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(prefix) {
                    files.push(path);
                }
            }
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

    let normalized = path_re
        .replace_all(&normalized, |caps: &regex::Captures| {
            let path = &caps[1];
            let basename = std::path::Path::new(path)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or(path);
            format!(r#""{basename}""#)
        })
        .into_owned();

    normalized
}

#[must_use]
pub fn normalize_timing_json_content(content: &str) -> String {
    let float_re = Regex::new("\\d+\\.\\d+").expect("hardcoded regex pattern is valid");
    let normalized = float_re.replace_all(content, "_IGNORED_").into_owned();
    normalized
}

/// Compare two files byte-by-byte
fn compare_files(expected: &Path, actual: &Path) -> Result<()> {
    let expected_bytes = std::fs::read(expected)
        .with_context(|| format!("Failed to read expected file: {}", expected.display()))?;
    let actual_bytes = std::fs::read(actual)
        .with_context(|| format!("Failed to read actual file: {}", actual.display()))?;

    // For JSON and HTML report files, normalize dynamic fields before comparison
    let (expected_normalized, actual_normalized) = if expected
        .extension()
        .is_some_and(|ext| ext == "json" || ext == "html")
    {
        let expected_str = String::from_utf8_lossy(&expected_bytes);
        let actual_str = String::from_utf8_lossy(&actual_bytes);

        let (expected_normalized, actual_normalized) = if expected
            .file_stem()
            .expect("path has extension so must have file_stem")
            .to_string_lossy()
            .ends_with("timing")
        {
            (
                normalize_timing_json_content(&expected_str),
                normalize_timing_json_content(&actual_str),
            )
        } else {
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
        if let Ok(parsed) = parsed {
            if let Some(step) = parsed.get("step") {
                if let Some(steps) = step.as_array() {
                    if let Some(step_no_i) = steps.get(step_int) {
                        if let Some(action) = step_no_i.get("action").and_then(|v| v.as_str()) {
                            return e.context(format!(
                                "Error in Step {step_no} (0-based), action = {action}"
                            ));
                        }
                    }
                }
            }
        }
    }
    e
}
