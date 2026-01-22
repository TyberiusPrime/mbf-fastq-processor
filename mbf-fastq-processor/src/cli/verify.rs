use anyhow::{Context, Result, bail};
use ex::fs;
use std::io::Write;
use std::path::Path;

use crate::cli;
use crate::config::Config;

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

    let expected_validation_error = cli::ExpectedFailure::new(&toml_dir, "error")?;
    let expected_validation_warning = cli::ExpectedFailure::new(&toml_dir, "validation_warning")?;
    let expected_runtime_error = cli::ExpectedFailure::new(&toml_dir, "runtime_error")?;

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

    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    let (output_prefix, uses_stdout) = {
        let parsed = eserde::toml::from_str::<Config>(&raw_config)
            .map_err(|e| cli::improve_error_messages(e.into(), &raw_config));

        if let Ok(parsed) = &parsed
            && let Some(benchmark) = &parsed.benchmark
            && benchmark.enable
        {
            bail!(
                "This is a benchmarking configuration, which can't be verified for it's output (it has none). Maybe turn off benchmark.enable in your TOML, or use another configuration?"
            )
        }

        parsed
            .ok()
            .and_then(|parsed| parsed.output.as_ref().map(|o| (o.prefix.clone(), o.stdout)))
            .unwrap_or_else(|| ("missing_output_config".to_string(), false))
    };

    let temp_dir = tempfile::tempdir().context("Failed to create temporary directory")?;
    let temp_path = if let Some(output_dir) = output_dir.as_ref() {
        if output_dir.exists() {
            ex::fs::remove_dir_all(&output_dir).with_context(|| {
                format!(
                    "Failed to remove existing output directory: {}",
                    output_dir.display()
                )
            })?;
        }
        std::fs::create_dir_all(&output_dir).with_context(|| {
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
                    cli::copy_input_file(value, &toml_dir, &temp_path)?;
                }
            }
        }
        if let Some(steps) = toml_value.get_mut("step").and_then(|v| v.as_array_mut()) {
            for step in steps {
                if let Some(step_table) = step.as_table_mut() {
                    for filename_key in ["filename", "filenames", "files"] {
                        if let Some(value) = step_table.get_mut(filename_key) {
                            cli::copy_input_file(value, &toml_dir, &temp_path)?;
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
                    cli::make_toml_path_absolute(value, &toml_dir);
                }
            }
        }

        if let Some(steps) = toml_value.get_mut("step").and_then(|v| v.as_array_mut()) {
            for step in steps {
                if let Some(step_table) = step.as_table_mut() {
                    for filename_key in ["filename", "filenames", "files"] {
                        if let Some(value) = step_table.get_mut(filename_key) {
                            cli::make_toml_path_absolute(value, &toml_dir);
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
        let warnings = crate::cli::validate::validate_config(&temp_toml_path).with_context(|| {
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
    }

    if test_script.exists() {
        let mut command = std::process::Command::new("bash");
        command
            .arg(test_script)
            .env("PROCESSOR_CMD", current_exe)
            .env("CONFIG_FILE", "config.toml")
            .env("NO_FRIENDLY_PANIC", "1")
            .current_dir(temp_path);

        let output = cli::run_command_with_timeout(&mut command).context("Failed to run test.sh")?;

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
                expected_failure_pattern.validate_expected_failure(&stderr)?;
            }
            (Some(_), true) => {
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
                        cli::find_output_files(expected_dir, &output_prefix).unwrap_or(Vec::new());

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

                        if let Err(e) = cli::compare_files(expected_file, &actual_file, expected_dir) {
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
                            cli::compare_files(&expected_stream_file, &actual_stream_file, expected_dir)
                        {
                            mismatches.push(format!("{stream_name}: {e}"));
                        }
                    } else if actual_stream_file.exists() {
                        mismatches.push(format!("Unexpected {stream_name} file"));
                    }
                }

                if !uses_stdout {
                    let actual_files = cli::find_output_files(&actual_dir, &output_prefix)?;
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
