//! Interactive mode for rapid development and testing of FASTQ processing pipelines
//!
//! This module provides an interactive mode that:
//! - Watches a TOML config file for changes
//! - Automatically prepends Head and `FilterReservoirSample` steps
//! - Appends an `Inspect` step to show results
//! - Adjusts paths and output settings for interactive use
//! - Displays results in a pretty format
//!

use anyhow::{Context, Result, bail};
use bstr::BString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};
use toml_edit::{DocumentMut, Item, Table, value};

fn interruptible_sleep(duration_ms: u64) {
    std::thread::sleep(Duration::from_millis(duration_ms));
}

/// Get current local time as a formatted string
fn get_local_time() -> String {
    use std::time::UNIX_EPOCH;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Issue getting system time into seconds since epoch")
        .as_secs();

    // Simple UTC time formatting (hours:minutes:seconds)
    let secs = now % 86400;
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{hours:02}:{minutes:02}:{seconds:02} UTC")
}

const DEFAULT_POLL_INTERVAL_MS: u64 = 1000;
const DEFAULT_HEAD_COUNT: u64 = 10_000;
const DEFAULT_SAMPLE_COUNT: u64 = 15;
const DEFAULT_INSPECT_COUNT: u64 = 15;

pub struct InteractiveConfig {
    pub head_count: u64,
    pub sample_count: u64,
    pub inspect_count: u64,
    pub max_runs: Option<u64>,
}

impl InteractiveConfig {
    #[must_use]
    pub fn new(
        head: Option<u64>,
        sample: Option<u64>,
        inspect: Option<u64>,
        max_runs: Option<u64>,
    ) -> Self {
        Self {
            head_count: head.unwrap_or(DEFAULT_HEAD_COUNT),
            sample_count: sample.unwrap_or(DEFAULT_SAMPLE_COUNT),
            inspect_count: inspect.unwrap_or(DEFAULT_INSPECT_COUNT),
            max_runs,
        }
    }
}

/// Runs the interactive mode, watching the specified TOML file for changes
pub fn run_interactive(
    toml_path: &Path,
    head: Option<u64>,
    sample: Option<u64>,
    inspect: Option<u64>,
    poll_interval_ms: Option<u64>,
    max_runs: Option<u64>,
) -> Result<()> {
    let poll_interval_ms = poll_interval_ms.unwrap_or(DEFAULT_POLL_INTERVAL_MS);
    let config = InteractiveConfig::new(head, sample, inspect, max_runs);

    println!("Interactive mode starting...");
    println!("Watching: {}", toml_path.display());
    println!("Polling every {poll_interval_ms}ms");
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
    let mut run_count: u64 = 0;
    //needs named output folder for test & user inspection
    let temp_dir =
        std::env::temp_dir().join(format!("fastqrab-interactive-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("Failed to create temp directory: {}", temp_dir.display()))?;

    // Remove the temp dir on Ctrl+C so it does not accumulate across runs.
    let cleanup_path = temp_dir.clone();
    ctrlc::set_handler(move || {
        let _ = fs::remove_dir_all(&cleanup_path);
        std::process::exit(0); // that's the expected behaviour
    })
    .context("Failed to register Ctrl+C handler")?;

    loop {
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
            run_count += 1;

            match process_toml_interactive(&temp_dir, &last_content, &toml_path, &config) {
                Ok(output) => {
                    display_success(&output);
                }
                Err(e) => {
                    display_error(&e);
                }
            }
            let _ = std::io::Write::flush(&mut std::io::stdout());

            if config.max_runs.is_some_and(|max| run_count >= max) {
                return Ok(());
            }
        } // cov:excl-line

        interruptible_sleep(poll_interval_ms);
    }
}

/// Process a TOML file in interactive mode
fn process_toml_interactive(
    temp_dir: &Path,
    content: &BString,
    toml_path: &Path,
    config: &InteractiveConfig,
) -> Result<String> {
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

    // Write modified TOML to temp directory
    let temp_toml = temp_dir.join("config.toml");
    let modified_content = doc.to_string();

    fs::write(&temp_toml, modified_content)
        .with_context(|| format!("Failed to write temp TOML: {}", temp_toml.display()))?;

    // Run the processor on the modified TOML
    let exe_path = std::env::current_exe().context("Failed to get current executable path")?;
    let output = Command::new(&exe_path)
        .arg("process")
        .arg("--allow-overwrite")
        .arg(&temp_toml)
        .current_dir(temp_dir)
        .output()
        .with_context(|| format!("Failed to execute: {}", exe_path.display()))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

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
        if inspect_file.exists()
            && let Ok(contents) = fs::read_to_string(&inspect_file)
        {
            inspect_output = contents;
        }

        // Combine for output
        let mut result = String::new();
        if !stdout.is_empty() {
            // cov:excl-start
            // normally, it's quiet on stdout...
            result.push_str(&stdout);
            // cov:excl-stop
        }
        if !inspect_output.is_empty() {
            if !result.is_empty() {
                // cov:excl-start
                //same stdout argument
                result.push_str("\n\n");
                // cov:excl-stop
            }
            result.push_str("Inspect Output:\n");
            result.push_str(&inspect_output);
        } // cov:excl-line
        Ok(result)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow::anyhow!("Processing failed:\n{stderr}"))
    }
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
    } // cov:excl-line

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
                        if let Some(s) = path_str.as_str() {
                            let path = PathBuf::from(s);
                            if !path.is_absolute() {
                                let absolute = toml_dir.join(path);
                                let absolute_str = absolute.to_string_lossy().to_string();
                                *path_str = absolute_str.into();
                            }
                        } // cov:excl-line
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

    // If there are Report steps, enable report_json
    if has_report_step {
        output_table.insert("report_json", value(true));
    }

    output_table.set_implicit(true);

    doc.insert("output", toml_edit::Item::Table(output_table));
}

/// Inject `Head`, `FilterReservoirSample` at start and Inspect at end of transform steps
fn inject_interactive_steps(doc: &mut DocumentMut, config: &InteractiveConfig) {
    // Create `Head` step table
    let mut head_table = Table::new();
    head_table.insert("action", value("Head"));
    head_table.insert(
        "n",
        value(i64::try_from(config.head_count).expect("Head count exceeded JSON number range i64")),
    );

    // Create `FilterReservoirSample` step table
    let mut sample_table = Table::new();
    sample_table.insert("action", value("FilterReservoirSample"));
    sample_table.insert(
        "n",
        value(
            i64::try_from(config.sample_count)
                .expect("sample count exceeded JSON number range i64"),
        ),
    );
    sample_table.insert("seed", value(42_i64));

    // Create Inspect step table
    let mut inspect_table = Table::new();
    inspect_table.insert("action", value("Inspect"));
    inspect_table.insert(
        "n",
        value(
            i64::try_from(config.inspect_count)
                .expect("inspect count exceeded JSON number range i64"),
        ),
    );
    inspect_table.insert("infix", value("inspect"));
    inspect_table.insert("segment", value("All"));

    // Get mutable reference to the step array and modify in place
    if let Some(step_item) = doc.get_mut("step")
        && let Some(array_of_tables) = step_item.as_array_of_tables_mut()
    {
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

        return;
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
#[expect(clippy::string_slice, reason = "just returned from find")]
fn display_success(output: &str) {
    println!("{}", "─".repeat(80));
    println!("Processing completed successfully [{}]", get_local_time());
    println!("{}", "─".repeat(80));

    // Find and highlight the Inspect output
    if let Some(inspect_start) = output.find("Inspect:") {
        // cov:excl-start
        let inspect_output = &output[inspect_start..];
        println!("\nSample Output:\n");
        println!("{inspect_output}");
    } else {
        // cov:excl-stop - yes this is the right place..
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn make_config(n: u64) -> InteractiveConfig {
        InteractiveConfig {
            head_count: n,
            sample_count: n,
            inspect_count: n,
            max_runs: None,
        }
    }

    fn parse_doc(toml: &str) -> DocumentMut {
        toml.parse::<DocumentMut>().expect("valid toml")
    }

    #[test]
    fn test_inject_steps_prepends_head_and_sample_and_appends_inspect() {
        let toml = r#"
[input]
read1 = ["a.fq"]

[[step]]
action = "Report"

[output]
prefix = "out"
"#;
        let mut doc = parse_doc(toml);
        let config = make_config(5);
        inject_interactive_steps(&mut doc, &config);

        let steps: Vec<_> = doc["step"]
            .as_array_of_tables()
            .expect("step array should exist")
            .iter()
            .collect();
        assert_eq!(
            steps.len(),
            4,
            "should have Head, FilterReservoirSample, Report, Inspect"
        );

        let action = |i: usize| steps[i]["action"].as_str().unwrap_or("").to_string();
        assert_eq!(action(0), "Head");
        assert_eq!(action(1), "FilterReservoirSample");
        assert_eq!(action(2), "Report");
        assert_eq!(action(3), "Inspect");

        // Check Head n value
        assert_eq!(steps[0]["n"].as_integer(), Some(5));
        // Check FilterReservoirSample seed
        assert_eq!(steps[1]["seed"].as_integer(), Some(42));
        // Check Inspect segment
        assert_eq!(steps[3]["segment"].as_str(), Some("All"));
    }

    #[test]
    fn test_inject_steps_creates_steps_when_none_exist() {
        let toml = r#"
[input]
read1 = ["a.fq"]

[output]
prefix = "out"
"#;
        let mut doc = parse_doc(toml);
        let config = make_config(10);
        inject_interactive_steps(&mut doc, &config);

        let steps: Vec<_> = doc["step"]
            .as_array_of_tables()
            .expect("step array should be created")
            .iter()
            .collect();
        assert_eq!(
            steps.len(),
            3,
            "should have Head, FilterReservoirSample, Inspect"
        );
        assert_eq!(steps[0]["action"].as_str(), Some("Head"));
        assert_eq!(steps[1]["action"].as_str(), Some("FilterReservoirSample"));
        assert_eq!(steps[2]["action"].as_str(), Some("Inspect"));
    }

    #[test]
    fn test_modify_output_sets_none_format() {
        let toml = r#"
[input]
read1 = ["a.fq"]

[[step]]
action = "Head"
n = 10

[output]
prefix = "original"
[[step]]
    action = 'outputfastq'
    compression = "Gzip"
"#;
        let mut doc = parse_doc(toml);
        modify_output_for_interactive(&mut doc);

        let output = doc["output"].as_table().expect("output table");
        assert_eq!(output["prefix"].as_str(), Some("interactive_output"));
        // report_json should NOT be set when there's no Report step
        assert!(output.get("report_json").is_none());
    }

    #[test]
    fn test_modify_output_enables_report_json_when_report_step_present() {
        let toml = r#"
[input]
read1 = ["a.fq"]

[[step]]
action = "Report"

[output]
prefix = "out"
"#;
        let mut doc = parse_doc(toml);
        modify_output_for_interactive(&mut doc);

        let output = doc["output"].as_table().expect("output table");
        assert_eq!(output["report_json"].as_bool(), Some(true));
    }

    #[test]
    fn test_make_paths_absolute_converts_relative_paths() {
        let toml = r#"
[input]
read1 = "relative/path.fq"
read2 = "other.fq"
"#;
        let mut doc = parse_doc(toml);
        let toml_dir = PathBuf::from("/some/directory");

        let input_table = doc["input"].as_table_mut().expect("input table");
        make_paths_absolute(input_table, &toml_dir).unwrap();

        let read1 = doc["input"]["read1"].as_str().expect("read1");
        assert!(read1.starts_with('/'), "should be absolute: {read1}");
        assert!(
            read1.contains("relative/path.fq"),
            "should contain original path: {read1}"
        );
    }

    #[test]
    fn test_make_paths_absolute_leaves_absolute_paths_unchanged() {
        let toml = r#"
[input]
read1 = "/already/absolute.fq"
"#;
        let mut doc = parse_doc(toml);
        let toml_dir = PathBuf::from("/some/directory");

        let input_table = doc["input"].as_table_mut().expect("input table");
        make_paths_absolute(input_table, &toml_dir).unwrap();

        let read1 = doc["input"]["read1"].as_str().expect("read1");
        assert_eq!(read1, "/already/absolute.fq");
    }

    #[test]
    fn test_interactive_config_defaults() {
        let config = InteractiveConfig::new(None, None, None, None);
        assert_eq!(config.head_count, DEFAULT_HEAD_COUNT);
        assert_eq!(config.sample_count, DEFAULT_SAMPLE_COUNT);
        assert_eq!(config.inspect_count, DEFAULT_INSPECT_COUNT);
    }

    #[test]
    fn test_interactive_config_custom_values() {
        let config = InteractiveConfig::new(Some(100), Some(5), Some(20), Some(10));
        assert_eq!(config.head_count, 100);
        assert_eq!(config.sample_count, 5);
        assert_eq!(config.inspect_count, 20);
        assert_eq!(config.max_runs, Some(10));
    }

    #[test]
    #[cfg(unix)]
    fn test_make_paths_absolute_handles_array_paths() {
        let toml = "[input]\nread1 = [\"a.fq\", \"b.fq\"]\n";
        let mut doc = parse_doc(toml);
        let toml_dir = PathBuf::from("/base");
        let input_table = doc["input"].as_table_mut().expect("input table");
        make_paths_absolute(input_table, &toml_dir).unwrap();
        // Serialize back and verify absolute paths appear
        let serialized = doc.to_string();
        assert!(
            serialized.contains("/base/a.fq"),
            "serialized: {serialized}"
        );
        assert!(
            serialized.contains("/base/b.fq"),
            "serialized: {serialized}"
        );
    }

    #[test]
    fn test_modify_toml_for_interactive_combines_all_modifications() {
        let toml = r#"
[input]
read1 = "relative.fq"

[[step]]
action = "Report"

[output]
prefix = "original"
[[step]]
    action = 'outputfastq'
    compression = "Gzip"
"#;
        let mut doc = parse_doc(toml);
        let toml_dir = PathBuf::from("/some/dir");
        let config = make_config(7);
        modify_toml_for_interactive(&mut doc, &toml_dir, &config).unwrap();

        // Paths made absolute
        let read1 = doc["input"]["read1"].as_str().expect("read1");
        assert!(read1.starts_with('/'), "path should be absolute: {read1}");

        // Steps injected: Head + FilterReservoirSample + original Report + Inspect = 4
        let steps: Vec<_> = doc["step"]
            .as_array_of_tables()
            .expect("steps")
            .iter()
            .collect();
        assert_eq!(steps.len(), 5);
        assert_eq!(steps[0]["action"].as_str(), Some("Head"));
        assert_eq!(steps[4]["action"].as_str(), Some("Inspect"));

        // Output overridden, report_json set because of Report step
        assert_eq!(doc["output"]["prefix"].as_str(), Some("interactive_output"));
        assert_eq!(doc["output"]["report_json"].as_bool(), Some(true));
    }

    #[test]
    fn test_process_toml_interactive_fails_on_invalid_utf8() {
        let invalid_utf8: BString = vec![0xFF, 0xFE, 0xFD].into();
        let toml_path = PathBuf::from("/some/dir/config.toml");
        let temp_dir = PathBuf::from("/tmp");
        let result =
            process_toml_interactive(&temp_dir, &invalid_utf8, &toml_path, &make_config(5));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("UTF-8") || msg.contains("utf8"), "err: {msg}");
    }

    #[test]
    fn test_process_toml_interactive_fails_on_invalid_toml() {
        let bad_toml: BString = b"this is [[[[ not valid toml".to_vec().into();
        let toml_path = PathBuf::from("/some/dir/config.toml");
        let temp_dir = PathBuf::from("/tmp");
        let result = process_toml_interactive(&temp_dir, &bad_toml, &toml_path, &make_config(5));
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("TOML") || msg.contains("toml") || msg.contains("parse"),
            "err: {msg}"
        );
    }
}
