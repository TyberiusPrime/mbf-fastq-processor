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

use crate::config::CheckedConfig;

#[allow(clippy::similar_names)] // I like rx/tx nomenclature
pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let parsed = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;
    let checked = parsed.check()?;
    let marker_prefix = checked
        .output
        .as_ref()
        .expect("config.check() ensures output is present")
        .prefix
        .clone();
    //only create the marker if we passed configuration validation
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.was_preexisting();

    let res = _run(
        checked,
        output_directory.as_ref(),
        allow_overwrite,
        raw_config,
    );

    match res {
        Ok(()) => {
            marker.mark_complete()?;
            Ok(())
        }
        Err(e) => {
            // if it's an already exist, we remove the marker
            if format!("{:?}", e).contains("already exists") {
                marker.mark_complete()?;
            }
            // otherwise, we leave it there to indicate incomplete run
            Err(e)
        }
    }
}

fn _run(
    mut parsed: CheckedConfig,
    output_directory: &Path,
    allow_overwrite: bool,
    raw_config: String,
) -> Result<()> {
    //parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
    let start_time = std::time::Instant::now();
    let is_benchmark = parsed.benchmark.as_ref().is_some_and(|b| b.enable);
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
        let run = run.create_output_threads(&parsed, raw_config)?;
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
    Ok(())
}

/// Validates a configuration file without requiring input files to exist
/// Returns Ok(warnings) if validation succeeds, Err if there are actual errors
pub fn validate_config(toml_file: &Path) -> Result<Vec<String>> {
    use std::fs;

    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let checked = eserde::toml::from_str::<Config>(&raw_config)
        .map_err(|e| improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?
        .check_for_validation()?;

    // Get the directory containing the TOML file to resolve relative paths
    let toml_dir = toml_file.parent().unwrap_or_else(|| Path::new("."));

    // Collect warnings about missing files after validation
    let mut warnings = Vec::new();

    // Check if input files exist and collect warnings
    match &checked.input.structured {
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

fn copy_input_file(value: &mut toml::Value, source_dir: &Path, target_dir: &Path) -> Result<()> {
    if let Some(path_str) = value.as_str() {
        if path_str != config::STDIN_MAGIC_PATH {
            let out_path = target_dir.join(path_str);
            let input_path = source_dir.join(path_str);
            //copy the file
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
                    if path_str == config::STDIN_MAGIC_PATH {
                        Ok(())
                    } else {
                        let out_path = target_dir.join(path_str);
                        let input_path = source_dir.join(path_str);
                        //copy the file
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
                    bail!("Invalid toml value")
                }
            })
            .collect();
        new_paths?;
    }
    Ok(())
}

/// Expected panic pattern types
enum ExpectedFailure {
    ExactText(String),
    Regex(Regex),
}

impl ExpectedFailure {
    /// Reads expected panic/error pattern from files in the given directory
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
            Ok(Some(ExpectedFailure::ExactText(content)))
        } else if expected_failure_regex_file.exists() {
            let content = ex::fs::read_to_string(&expected_failure_regex_file)
                .context("Read expected failure regex file")?
                .trim()
                .to_string();
            let regex = Regex::new(&content).context("Compile expected failure regex failed")?;
            Ok(Some(ExpectedFailure::Regex(regex)))
        } else {
            Ok(None)
        }
    }

    /// Validates that stderr matches the expected panic pattern
    fn validate_expected_failure(&self, stderr: &str) -> Result<()> {
        match self {
            ExpectedFailure::ExactText(expected_text) => {
                if !stderr.contains(expected_text) {
                    bail!(
                        "mbf-fastq-processor did not fail in the way that was expected.\nExpected message (substring): {}\nActual stderr: \n{}",
                        expected_text,
                        stderr
                    );
                }
            }
            ExpectedFailure::Regex(expected_regex) => {
                if !expected_regex.is_match(stderr) {
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

/// Verifies that running the configuration produces outputs matching expected outputs
/// in the directory where the TOML file is located
#[allow(clippy::too_many_lines)]
pub fn verify_outputs(
    toml_file: &Path,
    output_dir: Option<&Path>,
    unsafe_prep: bool,
) -> Result<()> {
    // Get the directory containing the TOML file
    let toml_file_abs = toml_file.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize TOML file path: {}",
            toml_file.display()
        )
    })?;
    let toml_dir = toml_file_abs.parent().unwrap_or_else(|| Path::new("."));
    let toml_dir = toml_dir.to_path_buf();

    let do_copy_input_files = toml_dir.join("copy_input").exists();

    // Check for expected panic files
    let expected_validation_error = ExpectedFailure::new(&toml_dir, "error")?;
    let expected_validation_warning = ExpectedFailure::new(&toml_dir, "validation_warning")?;
    let expected_runtime_error = ExpectedFailure::new(&toml_dir, "runtime_error")?;

    let error_file_count =
        expected_validation_error.is_some() as u8 + expected_runtime_error.is_some() as u8;
    if error_file_count > 1 {
        bail!(
            "Both expected_error(.txt|.regex) and expected_runtime_error(.txt|.regex) files exist. Please provide only one, depending on wether it's a validation or a processing error."
        );
    }

    let expected_failure = match (
        expected_validation_error.as_ref(),
        expected_runtime_error.as_ref(),
    ) {
        (Some(x), None) => Some(x),
        (None, Some(x)) => Some(x),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!(),
    };
    // Read the original TOML content
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    // Parse the TOML to extract configuration
    //
    let (output_prefix, uses_stdout) = {
        let parsed = eserde::toml::from_str::<Config>(&raw_config)
            .map_err(|e| improve_error_messages(e.into(), &raw_config));

        if let Ok(parsed) = &parsed
            && let Some(benchmark) = &parsed.benchmark
            && benchmark.enable
        {
            bail!(
                "This is a benchmarking configuration, which can't be verified for it's output (it has none). Maybe turn off benchmark.enable in your TOML, or use another configuration?"
            )
        }

        // Get the output configuration
        parsed
            .ok()
            .and_then(|parsed| parsed.output.as_ref().map(|o| (o.prefix.clone(), o.stdout)))
            //we do a default config so we get the full parsing
            .unwrap_or_else(|| ("missing_output_config".to_string(), false))
    };

    // Create a temporary directory for running the test
    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = temp_dir.path();

    // Parse the TOML as a generic toml::Value so we can modify it
    let mut toml_value: toml::Value =
        toml::from_str(&raw_config).context("Failed to parse TOML for modification")?;

    // Convert input file paths to absolute paths
    if do_copy_input_files {
        //copy all input files
        if let Some(input_table) = toml_value.get_mut("input").and_then(|v| v.as_table_mut()) {
            // Handle different input file fields
            let field_names: Vec<String> = input_table.keys().cloned().collect();
            for field_name in &field_names {
                if field_name == "interleaved" || field_name == "options" {
                    continue; // handled separately
                }
                if let Some(value) = input_table.get_mut(field_name) {
                    copy_input_file(value, &toml_dir, temp_path)?;
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
                            copy_input_file(value, &toml_dir, temp_path)?;
                        }
                    }
                }
            }
        }
    } else {
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

    // Handle prep.sh script if present and allowed - do this BEFORE parsing paths
    let prep_script = toml_dir.join("prep.sh");
    let post_script = toml_dir.join("post.sh");
    if unsafe_prep {
        if prep_script.exists() {
            // Run prep.sh script from original location but with temp directory as working directory
            #[cfg(not(target_os = "windows"))]
            let mut prep_command = {
                let mut command = std::process::Command::new("bash");
                command
                    .arg(prep_script.canonicalize().context("canonicalize prep.sh")?)
                    .current_dir(temp_path);
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
    }

    // Run processing in the temp directory
    // capture stdout & stderr - this means we must run ourselves as an external command!
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

    if expected_validation_error.is_none() | expected_validation_warning.is_some() {
        //we fully expect this to validate
        let warnings = validate_config(&temp_toml_path).with_context(|| {
            if expected_runtime_error.is_some() {
                "Configuration validation failed, but a runtime error was expected.".to_string()
            } else {
                "Configuration validation failed unexpectedly.".to_string()
            }
        })?;
        if let Some(expected_warning) = expected_validation_warning {
            if warnings.is_empty() {
                bail!("Expected validation warning, but none were produced.");
            } else {
                if !warnings
                    .iter()
                    .any(|w| expected_warning.validate_expected_failure(w).is_ok())
                {
                    bail!(
                        "Validation warnings did not match expected pattern.\nExpected: {}\nActual warnings:\n{}",
                        expected_warning,
                        warnings.join("\n")
                    );
                }
            }
        }
        // if !warnings.is_empty() {
        //     bail!(
        //         "Configuration produced validation warnings (and verify thus errors):\n{}",
        //         warnings.join("\n")
        //     );
        // }
    }

    let mut command = std::process::Command::new(current_exe);
    command
        .arg(if expected_validation_error.is_none() {
            "process"
        } else {
            "validate"
        })
        // .arg(&temp_toml_path)
        .arg("config.toml")
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

    // Handle execution result based on expected panic status
    match (expected_failure.as_ref(), output.status.success()) {
        (Some(expected_failure_pattern), false) => {
            // Expected panic and command failed - validate panic pattern
            expected_failure_pattern.validate_expected_failure(&stderr)?;
            // For panic tests, we don't need to compare outputs - just return success if the panic
            // occured as expected
            return Ok(());
        }
        (Some(_), true) => {
            // Expected panic but command succeeded
            if expected_validation_error.is_some() {
                bail!(
                    "Expected validation failure but 'validate' command succeeded. stderr: {}",
                    stderr
                );
            } else {
                bail!(
                    "Expected runtime failure but 'process' command succeeded. stderr: {}",
                    stderr
                );
            };
        }
        (None, false) => {
            // No panic expected but command failed
            bail!(
                "Processing failed with exit code {:?}. stderr: {}",
                output.status.code(),
                stderr
            );
        }
        (None, true) => {
            // No panic expected and command succeeded - continue with output comparison
        }
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

    let mut mismatches = Vec::new();
    //run post script first, it might manipulate the output files
    if post_script.exists() && unsafe_prep {
        // Run post.sh script from original location but with temp directory as working directory
        #[cfg(not(target_os = "windows"))]
        let mut post_command = {
            let mut command = std::process::Command::new("bash");
            command
                .arg(post_script.canonicalize().context("canonicalize post.sh")?)
                .current_dir(temp_path);
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

    // Compare outputs
    let expected_dir = &toml_dir;
    let actual_dir = temp_path;

    // Compare each output file (skip if output goes to stdout)
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
            copy_with_normalization(actual_dir, output_dir)?;
        }

        bail!("Output verification failed:\n  {}", mismatches.join("\n  "));
    }

    Ok(())
}

#[mutants::skip] // not called in normal operation, only when tests fail
fn copy_with_normalization(actual_dir: &Path, output_dir: &Path) -> Result<()> {
    // Copy all files from tempdir to output_dir with normalizers applied
    for entry in ex::fs::read_dir(actual_dir)
        .with_context(|| format!("Failed to read temp directory: {}", actual_dir.display()))?
    {
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
                let content = ex::fs::read_to_string(&src_path)
                    .with_context(|| format!("Failed to read file: {}", src_path.display()))?;

                let normalized = if src_path.extension().is_some_and(|ext| ext == "progress") {
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

    let content = normalize_re
        .replace_all(content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    // normalize numeric fields
    let normalize_re = Regex::new(r#""(?P<key>threads_per_segment|thread_count)"\s*:\s*[^"]*"#)
        .expect("invalid normalize regex");

    let content = normalize_re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("\"{}\": \"_IGNORED_\"", &caps["key"])
        })
        .into_owned();

    // Normalize input_toml field (contains full TOML with absolute paths)
    let input_toml_re =
        Regex::new(r#""input_toml"\s*:\s*"(?:[^"\\]|\\.)*""#).expect("invalid input_toml regex");

    let content = input_toml_re
        .replace_all(&content, r#""input_toml": "_IGNORED_""#)
        .into_owned();

    // Normalize file paths - convert absolute paths to basenames
    // This handles paths in input_files section
    let path_re = Regex::new(r#""(/[^"]+)""#).expect("invalid path regex");

    path_re
        .replace_all(&content, |caps: &regex::Captures| {
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
    let file_re =
        Regex::new("(?:^|[^A-Za-z0-9._-])(/(?:[^/\\s]+/)*([^/\\s]+))").expect("invalid file regex");
    file_re.replace_all(&normalized, "$2").into_owned()
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

fn calculate_size_difference_percent(len_a: u64, len_b: u64) -> f64 {
    if len_a > 0 {
        ((len_b as f64 - len_a as f64).abs() / len_a as f64) * 100.0
    } else if len_b > 0 {
        100.0 // One is empty, one isn't
    } else {
        0.0 // Both are empty
    }
}

/// Compare two files byte-by-byte
#[allow(clippy::cast_precision_loss)]
fn compare_files(expected: &Path, actual: &Path) -> Result<()> {
    // Check if these are compressed files
    let is_compressed = is_compressed_file(expected);

    let (expected_bytes, actual_bytes) = if is_compressed {
        // For compressed files, compare uncompressed content
        let expected_uncompressed = decompress_file(expected)?;
        let actual_uncompressed = decompress_file(actual)?;

        // Check that compressed file sizes are within 5% of each other
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

#[must_use]
#[mutants::skip] // Does not change output.
pub fn get_number_of_cores() -> usize {
    //get NUM_CPUS from env, or default to num_cpus
    std::env::var("MBF_FASTQ_PROCESSOR_NUM_CPUS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get())
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

        // Create a temporary gzip compressed file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        {
            let mut encoder =
                flate2::write::GzEncoder::new(&mut temp_file, flate2::Compression::default());
            encoder
                .write_all(b"Hello, world!")
                .expect("Failed to write to encoder");
            encoder.finish().expect("Failed to finish encoding");
        }

        // Decompress the file using the function
        let decompressed_data =
            decompress_file(temp_file.path()).expect("Failed to decompress file");

        // Verify the decompressed content
        assert_eq!(decompressed_data, b"Hello, world!");
    }
}
