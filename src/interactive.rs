//! Interactive mode for rapid development and testing of FastQ processing pipelines
//!
//! This module provides an interactive mode that:
//! - Watches a TOML config file for changes
//! - Automatically prepends Head and FilterReservoirSample steps
//! - Appends an Inspect step to show results
//! - Adjusts paths and output settings for interactive use
//! - Displays results in a pretty format

use anyhow::{bail, Context, Result};
use bstr::BString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration};
use toml_edit::{value, DocumentMut, Item, Table};

const POLL_INTERVAL_MS: u64 = 1000;
const HEAD_COUNT: i64 = 10_000;
const SAMPLE_COUNT: i64 = 15;
const INSPECT_COUNT: i64 = 15;

/// Runs the interactive mode, watching the specified TOML file for changes
pub fn run_interactive(toml_path: &Path) -> Result<()> {
    println!("Interactive mode starting...");
    println!("Watching: {}", toml_path.display());
    println!(" Polling every {}ms", POLL_INTERVAL_MS);
    println!("Will show first {} reads after sampling", INSPECT_COUNT);
    println!("\n{}", "=".repeat(80));
    println!("Press Ctrl+C to exit\n");

    let toml_path = toml_path
        .canonicalize()
        .with_context(|| format!("Failed to canonicalize path: {}", toml_path.display()))?;

    let mut last_content = b"".into();
    let mut first_run = true;

    loop {
        // Check if file has been modified

        let content: BString = fs::read(&toml_path)
            .with_context(|| format!("Failed to read file: {}", toml_path.display()))?
            .into();

        if first_run || content > last_content {
            last_content = content;

            if !first_run {
                println!("\n{}", "=".repeat(80));
                println!("🔄 File change detected, reprocessing...");
                println!("{}\n", "=".repeat(80));
            }
            first_run = false;

            match process_toml_interactive(&last_content, &toml_path) {
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
fn process_toml_interactive(content: &BString, toml_path: &Path) -> Result<String> {
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
    modify_toml_for_interactive(&mut doc, toml_dir)?;
    println!("{}", &doc.to_string());

    // Create temp directory
    let temp_dir =
        std::env::temp_dir().join(format!("mbf-fastq-interactive-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

    // Write modified TOML to temp directory
    let temp_toml = temp_dir.join("config.toml");
    fs::write(&temp_toml, doc.to_string())
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

        //list all files in tempdir
        for entry in fs::read_dir(&temp_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let filesize = fs::metadata(&path)?.len();
                println!("Generated file: {} {}", path.display(), filesize);
            }
        }

        // Look for the Inspect output file
        let mut inspect_output = String::new();
        let inspect_file = temp_dir.join("interactive_output_inspect_read1.fq");
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
        Err(anyhow::anyhow!("Processing failed:\n{}", stderr))
    };

    // Clean up temp directory
    let _ = fs::remove_dir_all(&temp_dir);

    result
}

/// Modify a TOML document for interactive mode
fn modify_toml_for_interactive(doc: &mut DocumentMut, toml_dir: &Path) -> Result<()> {
    // 1. Make input paths absolute
    if let Some(input_table) = doc.get_mut("input").and_then(|v| v.as_table_mut()) {
        make_paths_absolute(input_table, toml_dir)?;
    }

    // 2. Inject Head and FilterReservoirSample at the beginning of steps
    // 3. Inject Inspect at the end of steps
    inject_interactive_steps(doc)?;

    // 4. Set output to minimal (after steps so we can check for Report steps)
    modify_output_for_interactive(doc)?;

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
fn modify_output_for_interactive(doc: &mut DocumentMut) -> Result<()> {
    // Check if there are any Report steps
    let has_report_step = doc
        .get("step")
        .and_then(|step_item| step_item.as_array_of_tables())
        .map(|steps| {
            steps.iter().any(|step| {
                step.get("action")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "Report")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

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

    Ok(())
}

/// Inject Head, FilterReservoirSample at start and Inspect at end of transform steps
fn inject_interactive_steps(doc: &mut DocumentMut) -> Result<()> {
    // Create Head step table
    let mut head_table = Table::new();
    head_table.insert("action", value("Head"));
    head_table.insert("n", value(HEAD_COUNT));

    // Create FilterReservoirSample step table
    let mut sample_table = Table::new();
    sample_table.insert("action", value("FilterReservoirSample"));
    sample_table.insert("n", value(SAMPLE_COUNT));
    sample_table.insert("seed", value(42_i64));

    // Create Inspect step table
    let mut inspect_table = Table::new();
    inspect_table.insert("action", value("Inspect"));
    inspect_table.insert("n", value(INSPECT_COUNT));
    inspect_table.insert("infix", value("inspect"));

    // Handle the case where we have [[step]] (array of tables) or no steps at all
    // We'll collect existing steps, then rebuild with our injected steps
    let mut existing_steps = Vec::new();

    // Check for existing steps as array of tables (most common: [[step]])
    if let Some(step_item) = doc.get("step") {
        if let Some(array_of_tables) = step_item.as_array_of_tables() {
            // Collect existing steps as tables
            for table in array_of_tables.iter() {
                existing_steps.push(table.clone());
            }
            // Remove the old step array
            doc.remove("step");
        } else if let Some(array) = step_item.as_array() {
            // Handle inline table arrays
            for val in array.iter() {
                if let Some(inline_table) = val.as_inline_table() {
                    let mut table = Table::new();
                    for (k, v) in inline_table.iter() {
                        table.insert(k, Item::Value(v.clone()));
                    }
                    existing_steps.push(table);
                }
            }
            doc.remove("step");
        }
    }

    // Check for "transform" key as well
    if let Some(transform_item) = doc.get("transform") {
        if let Some(array_of_tables) = transform_item.as_array_of_tables() {
            for table in array_of_tables.iter() {
                existing_steps.push(table.clone());
            }
            doc.remove("transform");
        } else if let Some(array) = transform_item.as_array() {
            for val in array.iter() {
                if let Some(inline_table) = val.as_inline_table() {
                    let mut table = Table::new();
                    for (k, v) in inline_table.iter() {
                        table.insert(k, Item::Value(v.clone()));
                    }
                    existing_steps.push(table);
                }
            }
            doc.remove("transform");
        }
    }

    // Now create a new array of tables with our injected steps
    let mut new_steps = toml_edit::ArrayOfTables::new();

    // Add our head and sample steps first
    new_steps.push(head_table);
    new_steps.push(sample_table);

    // Add existing steps
    for step in existing_steps {
        new_steps.push(step);
    }

    // Add inspect step at the end
    new_steps.push(inspect_table);

    // Insert the new array of tables back into the document
    doc.insert("step", toml_edit::Item::ArrayOfTables(new_steps));

    Ok(())
}

/// Display successful processing results
fn display_success(output: &str) {
    println!("{}", "─".repeat(80));
    println!("✅ Processing completed successfully");
    println!("{}", "─".repeat(80));

    // Find and highlight the Inspect output
    if let Some(inspect_start) = output.find("Inspect:") {
        let inspect_output = &output[inspect_start..];
        println!("\n📊 Sample Output:\n");
        println!("{}", inspect_output);
    } else {
        // If no Inspect found, show all output
        if !output.trim().is_empty() {
            println!("\n📊 Output:\n");
            println!("{}", output);
        } else {
            println!("\n✓ No output (processing completed without messages)");
        }
    }

    println!("\n{}", "─".repeat(80));
}

/// Display error information
fn display_error(error: &anyhow::Error) {
    println!("{}", "─".repeat(80));
    println!("❌ Processing failed");
    println!("{}", "─".repeat(80));
    println!("\n{:?}", error);
    println!("\n{}", "─".repeat(80));
}
