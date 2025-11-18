#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

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
mod async_pipeline;
mod coordinator_pipeline;
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
    let marker_prefix = parsed.output.as_ref().unwrap().prefix.clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.preexisting();
    //parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
    #[allow(clippy::if_not_else)]
    match parsed.options.pipeline_mode {
        config::PipelineMode::Async => {
            // Async pipeline implementation using Tokio
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let run = async_pipeline::AsyncRunStage0::new(&parsed);
                let run = run.configure_demultiplex_and_init_stages(
                    &mut parsed,
                    &output_directory,
                    allow_overwrite,
                )?;
                let run = run.create_input_tasks(&parsed).await?;
                let run = run.create_stage_tasks(&mut parsed);
                let parsed = parsed; //after this, stages are transformed and ready, and config is read only.
                let run = run.create_output_task(&parsed, report_labels, raw_config)?;
                let run = run.join_tasks().await;

                let errors = run.errors;

                if !errors.is_empty() {
                    bail!(errors.join("\n"));
                }

                // Display timing information only after confirming no errors
                if !run.timings.is_empty() {
                    let stats = timing::aggregate_timings(run.timings);
                    let table = timing::format_timing_table(&stats);
                    eprintln!("\n\nPipeline Timing Statistics:");
                    eprintln!("{}", table);
                }

                drop(parsed);
                Ok::<(), anyhow::Error>(())
            })?;
        }
        config::PipelineMode::Coordinator => {
            // Coordinator-based pipeline with work pool
            let run = coordinator_pipeline::CoordRunStage0::new(&parsed);
            let run = run.configure_demultiplex_and_init_stages(
                &mut parsed,
                &output_directory,
                allow_overwrite,
            )?;
            let run = run.create_input_threads(&parsed)?;
            let run = run.create_coordinator(&mut parsed);
            let parsed = parsed; //after this, stages are transformed and ready, and config is read only.
            let run = run.create_output_thread(&parsed, report_labels, raw_config)?;
            let run = run.join_threads();

            let errors = run.errors;

            if !errors.is_empty() {
                bail!(errors.join("\n"));
            }

            // Display timing information only after confirming no errors
            if !run.timings.is_empty() {
                let stats = timing::aggregate_timings(run.timings);
                let table = timing::format_timing_table(&stats);
                eprintln!("\n\nPipeline Timing Statistics:");
                eprintln!("{}", table);
            }

            drop(parsed);
        }
        config::PipelineMode::ThreadBased => {
            // Original thread-based pipeline implementation
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

            // Display timing information only after confirming no errors
            if !run.timings.is_empty() {
                let stats = timing::aggregate_timings(run.timings);
                let table = timing::format_timing_table(&stats);
                eprintln!("\n\nPipeline Timing Statistics:");
                eprintln!("{}", table);
            }
            //assert!(errors.is_empty(), "Error in threads occured: {errors:?}");

            //ok all this needs is a buffer that makes sure we reorder correctly at the end.
            //and then something block based, not single reads to pass between the threads.
            drop(parsed);
        }
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
                    if !fs::metadata(&file_path).is_ok() {
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
                        if !fs::metadata(&file_path).is_ok() {
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
    let step_regex = Regex::new(r"step.(\d+).").unwrap();
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
                                "Error in Step {step_no} (1-based), action = {action}"
                            ));
                        }
                    }
                }
            }
        }
    }
    e
}
