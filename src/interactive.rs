//! Interactive mode for rapid development and testing of FASTQ processing pipelines
//!
//! This module provides an interactive mode that:
//! - Watches a TOML config file for changes
//! - Automatically prepends Head and `FilterReservoirSample` steps
//! - Appends an `Inspect` step to show results
//! - Adjusts paths and output settings for interactive use
//! - Displays results in a pretty format

use anyhow::{bail, Context, Result};
use bstr::BString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use toml_edit::{value, DocumentMut, Item, Table};

/// Get current local time as a formatted string
fn get_local_time() -> String {
    use std::time::UNIX_EPOCH;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple UTC time formatting (hours:minutes:seconds)
    let secs = now % 86400;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02} UTC")
}

const POLL_INTERVAL_MS: u64 = 1000;
const DEFAULT_HEAD_COUNT: u64 = 10_000;
const DEFAULT_SAMPLE_COUNT: u64 = 15;
const DEFAULT_INSPECT_COUNT: u64 = 15;

pub struct InteractiveConfig {
    pub head_count: u64,
    pub sample_count: u64,
    pub inspect_count: u64,
}

impl InteractiveConfig {
    #[must_use]
    pub fn new(head: Option<u64>, sample: Option<u64>, inspect: Option<u64>) -> Self {
        Self {
            head_count: head.unwrap_or(DEFAULT_HEAD_COUNT),
            sample_count: sample.unwrap_or(DEFAULT_SAMPLE_COUNT),
            inspect_count: inspect.unwrap_or(DEFAULT_INSPECT_COUNT),
        }
    }
}

/// Runs the interactive mode, watching the specified TOML file for changes
pub fn run_interactive(
    toml_path: &Path,
    head: Option<u64>,
    sample: Option<u64>,
    inspect: Option<u64>,
) -> Result<()> {
    let config = InteractiveConfig::new(head, sample, inspect);

    println!("Interactive mode starting...");
    println!("Watching: {}", toml_path.display());
    println!("Polling every {POLL_INTERVAL_MS}ms");
    println!("Processing first {} reads", config.head_count);
    println!("Sampling {} reads for display", config.sample_count);
    println!("Showing {} reads in output", config.inspect_count);
    println!("\n{}", "=".repeat(80));
    println!("Press Ctrl+C to exit\n");

    let toml_path = toml_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", toml_path.display()))?;

    let mut last_content = b"".into();
    let mut first_run = true;
    let temp_dir =
        std::env::temp_dir().join(format!("mbf-fastq-interactive-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

    loop {
        // Check if file has been modified

        let content: BString = fs::read(&toml_path)
            .with_context(|| format!("Failed to read file: {}", toml_path.display()))?
            .into();

        if first_run || content != last_content {
            last_content = content;

            if !first_run {
                println!("\n{}", "=".repeat(80));
                println!("🔄 File change detected, reprocessing...");
                println!("{}\n", "=".repeat(80));
            }
            first_run = false;

            match process_toml_interactive(&temp_dir, &last_content, &toml_path, &config) {
                Ok(output) => {
                    display_success(&output);
                }
                Err(e) => {
                    display_error(&e);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
    }
}

/// Process a TOML file in interactive mode
fn process_toml_interactive(
    temp_dir: &Path,
    content: &BString,
    toml_path: &Path,
    config: &InteractiveConfig,
) -> Result<String> {
    // Read the original TOML content

    // Parse as toml_edit document to preserve formatting
    let mut doc = std::str::from_utf8(content)
        .context("UTF-8 error")?
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;

    // Get the directory containing the TOML file for resolving relative paths
    let toml_dir = toml_path
        .parent()
        .context("Failed to get parent directory")?;

    // Modify the document
    modify_toml_for_interactive(&mut doc, toml_dir, config)?;
    //println!("{}", &doc.to_string());

    // Create temp directory

    // Write modified TOML to temp directory
    let temp_toml = temp_dir.join("config.toml");
    let modified_content = doc.to_string();

    // Debug: print the modified TOML for inspection
    /* eprintln!("\n=== Modified TOML ===");
    eprintln!("{}", modified_content);
    eprintln!("=== End Modified TOML ===\n"); */
    eprintln!("Temp dir: {}", temp_dir.display());

    fs::write(&temp_toml, modified_content)
        .with_context(|| format!("Failed to write temp TOML: {}", temp_toml.display()))?;

    // Get the current executable path
    let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

    // Run the processor on the modified TOML
    let output = Command::new(&exe_path)
        .arg("process")
        .arg(&temp_toml)
        .current_dir(&temp_dir)
        .output()
        .with_context(|| format!("Failed to execute: {}", exe_path.display()))?;

    let result = if output.status.success() {
        // Extract stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        /* //list all files in tempdir
        for entry in fs::read_dir(&temp_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let filesize = fs::metadata(&path)?.len();
                println!("Generated file: {} {}", path.display(), filesize);
            }
        } */

        // Look for the Inspect output file
        let mut inspect_output = String::new();
        let inspect_file = temp_dir.join("interactive_output_inspect_interleaved.fq");
        if inspect_file.exists() {
            if let Ok(contents) = fs::read_to_string(&inspect_file) {
                inspect_output = contents;
            }
        }

        // Combine for output
        let mut result = String::new();
        if !stderr.is_empty() {
            result.push_str(&stderr);
        }
        if !stdout.is_empty() {
            if !result.is_empty() {
                result.push('\n');
            }
            result.push_str(&stdout);
        }
        if !inspect_output.is_empty() {
            if !result.is_empty() {
                result.push_str("\n\n");
            }
            result.push_str("Inspect Output:\n");
            result.push_str(&inspect_output);
        }
        Ok(result)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Processing failed:\n{stderr}"))
    };

    result
}

/// Modify a TOML document for interactive mode
fn modify_toml_for_interactive(
    doc: &mut DocumentMut,
    toml_dir: &Path,
    config: &InteractiveConfig,
) -> Result<()> {
    // 1. Make input paths absolute
    if let Some(input_table) = doc.get_mut("input").and_then(|v| v.as_table_mut()) {
        make_paths_absolute(input_table, toml_dir)?;
    }

    // 2. Inject Head and FilterReservoirSample at the beginning of steps
    // 3. Inject Inspect at the end of steps
    inject_interactive_steps(doc, config);

    // 4. Set output to minimal (after steps so we can check for Report steps)
    modify_output_for_interactive(doc);

    Ok(())
}

/// Make all file paths in the input section absolute
fn make_paths_absolute(input_table: &mut Table, toml_dir: &Path) -> Result<()> {
    for (key, value) in input_table.iter_mut() {
        if key == "options" || key == "interleaved" {
            continue;
        }
        match value {
            Item::Value(value) => match value {
                toml_edit::Value::Array(array) => {
                    for path_str in array.iter_mut() {
                        let path = PathBuf::from(path_str.to_string());
                        if !path.is_absolute() {
                            let absolute = toml_dir.join(path);
                            let absolute_str = absolute.to_string_lossy().to_string();
                            *path_str = absolute_str.into();
                        }
                    }
                }
                toml_edit::Value::String(path_str) => {
                    let path = PathBuf::from(path_str.value().as_str());
                    if !path.is_absolute() {
                        let absolute = toml_dir.join(path);
                        let absolute_str = absolute.to_string_lossy().to_string();
                        *path_str = toml_edit::Formatted::new(absolute_str);
                    }
                }
                _ => bail!("Input section unparsable, segment values not arrays or strings"),
            },
            _ => bail!("Input section unparsable"),
        }
    }
    Ok(())
}

/// Modify output section for interactive mode
fn modify_output_for_interactive(doc: &mut DocumentMut) {
    // Check if there are any Report steps
    let has_report_step = doc
        .get("step")
        .and_then(|step_item| step_item.as_array_of_tables())
        .is_some_and(|steps| {
            steps.iter().any(|step| {
                step.get("action")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s == "Report")
            })
        });

    // Create a minimal output configuration
    let mut output_table = Table::new();
    output_table.insert("prefix", value("interactive_output"));
    output_table.insert("format", value("none"));

    // If there are Report steps, enable report_json
    if has_report_step {
        output_table.insert("report_json", value(true));
    }

    output_table.set_implicit(true);

    doc.insert("output", toml_edit::Item::Table(output_table));
}

/// Inject `Head`, `FilterReservoirSample` at start and Inspect at end of transform steps
#[allow(clippy::cast_possible_wrap)]
fn inject_interactive_steps(doc: &mut DocumentMut, config: &InteractiveConfig) {
    // Create `Head` step table
    let mut head_table = Table::new();
    head_table.insert("action", value("Head"));
    head_table.insert("n", value(config.head_count as i64));

    // Create `FilterReservoirSample` step table
    let mut sample_table = Table::new();
    sample_table.insert("action", value("FilterReservoirSample"));
    sample_table.insert("n", value(config.sample_count as i64));
    sample_table.insert("seed", value(42_i64));

    // Create Inspect step table
    let mut inspect_table = Table::new();
    inspect_table.insert("action", value("Inspect"));
    inspect_table.insert("n", value(config.inspect_count as i64));
    inspect_table.insert("infix", value("inspect"));
    inspect_table.insert("segment", value("All"));

    // Get mutable reference to the step array and modify in place
    if let Some(step_item) = doc.get_mut("step") {
        if let Some(array_of_tables) = step_item.as_array_of_tables_mut() {
            // Prepend head and sample tables at the beginning
            // Note: ArrayOfTables doesn't have insert, so we need to rebuild
            let mut existing_steps = Vec::new();
            for table in array_of_tables.iter() {
                existing_steps.push(table.clone());
            }

            // Clear the array
            array_of_tables.clear();

            // Add head and sample first
            array_of_tables.push(head_table);
            array_of_tables.push(sample_table);

            // Re-add existing steps
            for step in existing_steps {
                array_of_tables.push(step);
            }

            // Add inspect at the end
            array_of_tables.push(inspect_table);

            return
        }
    }

    // If no step array exists, create one with our injected steps
    let mut new_steps = toml_edit::ArrayOfTables::new();
    new_steps.push(head_table);
    new_steps.push(sample_table);
    new_steps.push(inspect_table);

    // Insert at the beginning of the document by prepending to root
    doc.insert("step", toml_edit::Item::ArrayOfTables(new_steps));

}

/// Display successful processing results
fn display_success(output: &str) {
    println!("{}", "─".repeat(80));
    println!("Processing completed successfully [{}]", get_local_time());
    println!("{}", "─".repeat(80));

    // Find and highlight the Inspect output
    if let Some(inspect_start) = output.find("Inspect:") {
        let inspect_output = &output[inspect_start..];
        println!("\nSample Output:\n");
        println!("{inspect_output}");
    } else {
        // If no Inspect found, show all output
        if output.trim().is_empty() {
            println!("\n✓ No output (processing completed without messages)");
        } else {
            println!("\n📊 Output:\n");
            println!("{output}");
        }
    }

    println!("\n{}", "─".repeat(80));
}

/// Display error information
fn display_error(error: &anyhow::Error) {
    println!("{}", "─".repeat(80));
    println!("Processing failed [{}]", get_local_time());
    println!("{}", "─".repeat(80));
    println!("\n{error:?}");
    println!("\n{}", "─".repeat(80));
}
