#![allow(clippy::identity_op)]

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

#[test]
fn test_cookbooks_in_sync() {
    // Verify that the generated cookbooks.rs matches the actual cookbook directories
    use std::collections::HashSet;
    use std::path::Path;

    // Get cookbooks from generated code
    let generated_cookbooks: HashSet<String> = mbf_fastq_processor::cookbooks::list_cookbooks()
        .iter()
        .map(|(_, name)| (*name).to_string())
        .collect();

    // Get cookbooks from filesystem
    let cookbooks_dir = Path::new("cookbooks");
    assert!(cookbooks_dir.exists(), "cookbooks directory should exist");

    //contents always matches since they"re include_str!()ed

    let mut fs_cookbooks = HashSet::new();
    if let Ok(entries) = std::fs::read_dir(cookbooks_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let input_toml = entry.path().join("input.toml");
                if input_toml.exists() {
                    if let Some(name) = entry.file_name().to_str() {
                        fs_cookbooks.insert(name.to_string());
                    }
                }
            }
        }
    }

    // Check that they match
    let missing_in_generated: Vec<_> = fs_cookbooks.difference(&generated_cookbooks).collect();
    let extra_in_generated: Vec<_> = generated_cookbooks.difference(&fs_cookbooks).collect();

    if !missing_in_generated.is_empty() || !extra_in_generated.is_empty() {
        eprintln!("\n❌ Cookbook synchronization mismatch!");
        if !missing_in_generated.is_empty() {
            eprintln!("  Missing in generated code: {missing_in_generated:?}");
        }
        if !extra_in_generated.is_empty() {
            eprintln!("  Extra in generated code: {extra_in_generated:?}");
        }
        eprintln!("\n  Run: python3 dev/updated_generated.sh");
        panic!("Cookbooks out of sync. Run dev/update_generated.sh to regenerate.");
    }
}

#[test]
fn test_usage() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        //.join("debug")
        .join("mbf-fastq-processor");
    let cmd = std::process::Command::new(bin_path).output().unwrap();
    //let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    assert!(stderr.contains("Usage:"));
    assert!(!cmd.status.success());
}

#[test]
fn test_process_command() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Test process command without config file - should show error
    let cmd = std::process::Command::new(&bin_path)
        .arg("process")
        .output()
        .unwrap();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    assert!(stderr.contains("Please specify a configuration file explicitly"));
    assert!(!cmd.status.success());
}

#[test]
fn test_template_command() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    let cmd = std::process::Command::new(bin_path)
        .arg("template")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    // Verify template contains key sections
    assert!(stdout.contains("# mbf-fastq-processor Configuration Template"));
    assert!(stdout.contains("[input]"));
    assert!(stdout.contains("[output]"));
    assert!(stdout.contains("[[step]]"));
    assert!(cmd.status.success());
}

#[test]
fn test_version_command() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    let cmd = std::process::Command::new(bin_path)
        .arg("version")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    // Verify version output contains version number
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(cmd.status.success());
}

#[test]
fn test_cookbook_command() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Test cookbook list
    let cmd = std::process::Command::new(&bin_path)
        .arg("cookbook")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    assert!(stdout.contains("Available cookbooks:"));
    assert!(stdout.contains("01-basic-quality-report"));
    assert!(cmd.status.success());

    // Test specific cookbook
    let cmd = std::process::Command::new(&bin_path)
        .arg("cookbook")
        .arg("1")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    //assert!(stdout.contains("Cookbook 1:"));
    assert!(stdout.contains("## Configuration"));
    assert!(cmd.status.success());
}

#[test]
fn test_list_steps_command() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    let cmd = std::process::Command::new(bin_path)
        .arg("list-steps")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    assert!(stdout.contains("Available transformation steps:"));
    assert!(stdout.contains("Report"));
    assert!(stdout.contains("Head"));
    assert!(cmd.status.success());
}

#[test]
fn test_version_flag() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    let cmd = std::process::Command::new(bin_path)
        .arg("--version")
        .output()
        .unwrap();
    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();

    // Verify --version flag produces same output as version command
    assert!(!stdout.trim().is_empty());
    assert!(stdout.contains(env!("CARGO_PKG_VERSION")));
    assert!(cmd.status.success());
}

fn scan_dir(dir: &Path, files: &mut HashSet<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_dir(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Check if file contains DemultiplexedData field declarations
                    // but skip if it's only imports/uses
                    let has_demux_field = content.lines().any(|line| {
                        let trimmed = line.trim();
                        trimmed.contains("DemultiplexedData<")
                            && !trimmed.contains("use ")
                            && !trimmed.starts_with("//")
                            && (trimmed.contains("pub ")
                                || trimmed.contains(": ")
                                || trimmed.ends_with("DemultiplexedData,"))
                    });

                    if has_demux_field {
                        files.insert(path);
                    }
                }
            }
        }
    }
}

fn find_toml_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_toml_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                files.push(path);
            }
        }
    }
}
#[test]
#[allow(clippy::too_many_lines)]
fn test_every_demultiplexed_data_transform_has_test() {
    // This test verifies that every transformation that uses DemultiplexedData
    // has at least one test case where it occurs after a Demultiplex step.
    // The list of transforms is automatically discovered by scanning the source code.

    // Step 1: Find all Rust files containing DemultiplexedData field declarations
    let mut files_with_demux = HashSet::new();

    scan_dir(Path::new("src/transformations"), &mut files_with_demux);

    // Step 2: Extract public struct names from these files (excluding internal ones)
    let mut struct_names = HashSet::new();
    for file_path in &files_with_demux {
        if let Ok(content) = fs::read_to_string(file_path) {
            for line in content.lines() {
                if line.contains("pub struct") && !line.contains("pub(crate)") {
                    if let Some(struct_part) = line.split("pub struct").nth(1) {
                        // Extract the name - it's the first word after "pub struct"
                        let name = struct_part
                            .trim()
                            .split(|c: char| c == '{' || c == '<' || c.is_whitespace())
                            .find(|s| !s.is_empty())
                            .unwrap_or("")
                            .to_string();

                        // Skip internal structs (starting with _)
                        if !name.is_empty() && !name.starts_with('_') {
                            struct_names.insert(name);
                        }
                    }
                }
            }
        }
    }

    // Step 3: Parse Transformation enum to map struct names to action names
    let transformations_path = Path::new("src/transformations.rs");
    let transformations_content =
        fs::read_to_string(transformations_path).expect("Failed to read src/transformations.rs");

    let mut struct_to_action: HashMap<String, String> = HashMap::new();

    // Find the enum definition and parse it
    let mut in_enum = false;
    for line in transformations_content.lines() {
        if line.contains("pub enum Transformation") {
            in_enum = true;
            continue;
        }

        if in_enum {
            if line.trim() == "}" {
                break;
            }

            // Skip lines with #[serde(skip)] or comments
            if line.contains("#[serde(skip)]") || line.trim().starts_with("//") {
                continue;
            }

            // Parse enum variants: ActionName(module::path::StructName)
            if let Some(variant) = line.trim().strip_suffix(',').or(Some(line.trim())) {
                if let Some((action_name, struct_path)) = variant.split_once('(') {
                    let action_name = action_name.trim();
                    let struct_path = struct_path.trim_end_matches(')').trim();

                    // Extract just the struct name from the path
                    if let Some(struct_name) = struct_path.split("::").last() {
                        // Handle Box<...> wrapper
                        let struct_name =
                            struct_name.trim_start_matches("Box<").trim_end_matches('>');

                        if struct_names.contains(struct_name) {
                            struct_to_action
                                .insert(struct_name.to_string(), action_name.to_string());
                        }
                    }
                }
            }
        }
    }

    // Get the set of action names that use DemultiplexedData
    let transforms_with_demultiplexed_data: HashSet<String> =
        struct_to_action.values().cloned().collect();

    assert!(
        !transforms_with_demultiplexed_data.is_empty(),
        "No transforms with DemultiplexedData found - this is likely a bug in the test"
    );

    // Step 4: Find all test TOML files
    let test_cases_dir = Path::new("test_cases");
    let mut toml_files = Vec::new();

    find_toml_files(test_cases_dir, &mut toml_files);

    // Step 5: Track which transforms have tests after Demultiplex
    let mut tested_transforms = HashSet::new();

    // Check each TOML file for Demultiplex followed by our transforms
    for toml_path in &toml_files {
        if let Ok(content) = std::fs::read_to_string(toml_path) {
            let lines: Vec<&str> = content.lines().collect();
            let mut found_demultiplex = false;

            for line in lines {
                let trimmed = line.trim();

                // Check for Demultiplex action
                if trimmed.contains("action")
                    && (trimmed.contains("'Demultiplex'") || trimmed.contains("\"Demultiplex\""))
                {
                    found_demultiplex = true;
                }

                // If we've seen a Demultiplex, check for our transforms
                if found_demultiplex && trimmed.contains("action") {
                    for transform in &transforms_with_demultiplexed_data {
                        if trimmed.contains(&format!("'{transform}'"))
                            || trimmed.contains(&format!("\"{transform}\""))
                        {
                            tested_transforms.insert(transform.clone());
                            println!(
                                "✓ Found test for transform '{transform}' after Demultiplex in {}",
                                toml_path.display()
                            );
                        }
                    }
                }
            }
        }
    }

    // Step 6: Check for missing tests
    let missing_tests: Vec<_> = transforms_with_demultiplexed_data
        .difference(&tested_transforms)
        .collect();

    if !missing_tests.is_empty() {
        eprintln!("\n❌ The following transforms use DemultiplexedData but have no test cases");
        eprintln!("   where they occur after a Demultiplex step:");
        for transform in &missing_tests {
            eprintln!("   - {transform}");
        }
        eprintln!("\n  Please add test cases in test_cases/demultiplex/ for these transforms.");
        panic!(
            "Missing demultiplex tests for {} transform(s)",
            missing_tests.len()
        );
    }

    // Print success message
    println!(
        "\n✓ All {} transforms with DemultiplexedData have tests after Demultiplex:",
        transforms_with_demultiplexed_data.len()
    );
    for transform in &transforms_with_demultiplexed_data {
        println!("  ✓ {transform}");
    }
}

#[test]
fn test_readme_toml_examples_validate() {
    // This test extracts TOML code blocks from README.md and validates them
    use mbf_fastq_processor::config::Config;
    use std::fs;
    use std::path::Path;

    let readme_path = Path::new("README.md");
    assert!(readme_path.exists(), "README.md not found");

    let readme_content = fs::read_to_string(readme_path).expect("Failed to read README.md");

    // Extract TOML code blocks (between ```toml and ```)
    let mut toml_blocks = Vec::new();
    let mut in_toml_block = false;
    let mut current_block = String::new();
    let mut block_start_line = 0;
    let mut line_num = 0;

    for line in readme_content.lines() {
        line_num += 1;
        if line.trim().starts_with("```toml") {
            in_toml_block = true;
            current_block.clear();
            block_start_line = line_num;
        } else if line.trim().starts_with("```") && in_toml_block {
            in_toml_block = false;
            if !current_block.trim().is_empty() {
                toml_blocks.push((block_start_line, current_block.clone()));
            }
        } else if in_toml_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    assert!(
        !toml_blocks.is_empty(),
        "No TOML code blocks found in README.md"
    );

    println!("\n✓ Found {} TOML block(s) in README.md", toml_blocks.len());

    // Validate each TOML block using the same approach as the run() function
    for (line_no, toml_content) in &toml_blocks {
        println!("  Validating TOML block starting at line {line_no}...");

        // Parse the TOML using eserde (same as in run())
        let mut parsed = match eserde::toml::from_str::<Config>(toml_content) {
            Ok(config) => config,
            Err(e) => {
                panic!("README.md TOML block at line {line_no} failed to parse:\n{e:?}",);
            }
        };

        // Validate the config using check() (same as in run())
        // Note: This will fail on input file validation since files don't exist,
        // but it will catch TOML syntax errors and structural issues
        match parsed.check() {
            Ok(()) => {
                println!("    ✓ TOML block at line {line_no} validated successfully",);
            }
            Err(e) => {
                let error_msg = format!("{e:?}");
                // Allow errors about missing input files, but catch everything else
                if error_msg.contains("Could not read")
                    || error_msg.contains("No such file")
                    || error_msg.contains("does not exist")
                {
                    println!(
                        "    ✓ TOML block at line {line_no} validated (structure valid, expected file errors ignored)",
                    );
                } else {
                    panic!(
                        "README.md TOML block at line {line_no} failed validation:\n{error_msg}",
                    );
                }
            }
        }
    }

    println!("\n✓ All README.md TOML examples are valid!");
}

/*
* difficult to test, since it only works in --release build binaries...
We're going to test it in the nix build, I suppose
#[test]
fn test_friendly_panic() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        //.join("debug")
        .join("mbf-fastq-processor");
    let cmd = std::process::Command::new(bin_path).arg("--test-friendly-panic").output().unwrap();
    //let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();
    assert!(stderr.contains("Usage:"));
    assert!(!cmd.status.success());
} */

#[test]
fn test_validate_command_valid_config_with_existing_files() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory and files
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq files
    let mut file1 = fs::File::create(temp_path.join("test1.fq")).unwrap();
    writeln!(file1, "@read1\nACGT\n+\nIIII").unwrap();

    let mut file2 = fs::File::create(temp_path.join("test2.fq")).unwrap();
    writeln!(file2, "@read2\nACGT\n+\nIIII").unwrap();

    // Create valid config
    let config_path = temp_path.join("valid_config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read_1 = 'test1.fq'
read_2 = 'test2.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true

[output]
prefix = 'output'
report_html = true
"
    )
    .unwrap();

    // Run validate command
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg(&config_path)
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stdout.contains("✓ Configuration is valid"),
        "Expected success message, got: {stdout}"
    );
    assert!(
        !stdout.contains("with warnings"),
        "Should not have warnings with existing files"
    );
    assert!(stderr.is_empty(), "Should have no warnings in stderr");
    assert!(cmd.status.success(), "Exit code should be 0");
}

#[test]
fn test_validate_command_valid_config_missing_files() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create config referencing non-existent files
    let config_path = temp_path.join("missing_files_config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read_1 = 'nonexistent1.fq'
read_2 = 'nonexistent2.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true

[output]
prefix = 'output'
report_html = true
"
    )
    .unwrap();

    // Run validate command
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg(&config_path)
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stdout.contains("✓ Configuration is valid (with warnings)"),
        "Expected success with warnings, got: {stdout}"
    );
    assert!(
        stderr.contains("Warning: Input file not found"),
        "Expected file not found warning in stderr: {stderr}"
    );
    assert!(
        stderr.contains("nonexistent1.fq") || stderr.contains("nonexistent2.fq"),
        "Expected missing file names in warnings"
    );
    assert!(
        cmd.status.success(),
        "Exit code should be 0 even with missing files"
    );
}

#[test]
fn test_validate_command_invalid_action() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create config with invalid action
    let config_path = temp_path.join("invalid_action.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read_1 = 'test.fq'

[[step]]
action = 'InvalidAction'
name = 'test'

[output]
prefix = 'output'
"
    )
    .unwrap();

    // Run validate command
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg(&config_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stderr.contains("Configuration validation failed"),
        "Expected validation failure message"
    );
    assert!(
        stderr.contains("InvalidAction") || stderr.contains("Unknown variant"),
        "Expected error about invalid action: {stderr}"
    );
    assert!(
        !cmd.status.success(),
        "Exit code should be non-zero for invalid config"
    );
}

#[test]
fn test_validate_command_nonexistent_toml() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Try to validate a non-existent file
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg("/nonexistent/path/to/config.toml")
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stderr.contains("Configuration validation failed") || stderr.contains("Could not read"),
        "Expected error about missing TOML file: {stderr}"
    );
    assert!(
        !cmd.status.success(),
        "Exit code should be non-zero for missing file"
    );
}

#[test]
fn test_validate_command_malformed_toml() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create malformed TOML
    let config_path = temp_path.join("malformed.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input
read_1 = 'test.fq'
this is not valid toml
"
    )
    .unwrap();

    // Run validate command
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg(&config_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stderr.contains("Configuration validation failed") || stderr.contains("Could not parse"),
        "Expected error about malformed TOML: {stderr}"
    );
    assert!(
        !cmd.status.success(),
        "Exit code should be non-zero for malformed TOML"
    );
}

#[test]
fn test_validate_command_missing_required_fields() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create config missing required fields (no output)
    let config_path = temp_path.join("missing_fields.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read_1 = 'test.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true
"
    )
    .unwrap();

    // Run validate command
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .arg(&config_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    // This should fail because output is required when there's a Report step
    assert!(
        stderr.contains("Configuration validation failed")
            || stderr.contains("Report")
            || stderr.contains("No output"),
        "Expected error about missing output configuration: {stderr}"
    );
    assert!(!cmd.status.success(), "Exit code should be non-zero");
}

#[test]
fn test_validate_command_no_arguments() {
    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Run validate without config file
    let cmd = std::process::Command::new(bin_path)
        .arg("validate")
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&cmd.stderr).unwrap().to_string();

    assert!(
        stderr.contains("required arguments were not provided") || stderr.contains("<CONFIG_TOML>"),
        "Expected error about missing config argument: {stderr}"
    );
    assert!(!cmd.status.success(), "Exit code should be non-zero");
}

#[test]
fn test_verify_command_matching_outputs() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq file
    let mut input_file = fs::File::create(temp_path.join("input.fq")).unwrap();
    writeln!(input_file, "@read1\nACGT\n+\nIIII").unwrap();
    writeln!(input_file, "@read2\nTGCA\n+\nIIII").unwrap();

    // Create config with JSON and HTML reports
    let config_path = temp_path.join("config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read1 = 'input.fq'

[[step]]
action = 'Head'
n = 1

[[step]]
action = 'Report'
name = 'test_report'
count = true

[output]
prefix = 'output'
report_json = true
report_html = true
"
    )
    .unwrap();

    // First, run process to generate expected outputs
    let process_cmd = std::process::Command::new(&bin_path)
        .arg("process")
        .arg(&config_path)
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(
        process_cmd.status.success(),
        "Process command should succeed: {}",
        std::str::from_utf8(&process_cmd.stderr).unwrap()
    );

    // Verify that output files were created
    assert!(
        temp_path.join("output_read1.fq").exists(),
        "Output fastq file should exist"
    );
    assert!(
        temp_path.join("output.json").exists(),
        "Output JSON report should exist"
    );
    assert!(
        temp_path.join("output.html").exists(),
        "Output HTML report should exist"
    );

    // Now run verify command - should pass since outputs match
    let verify_cmd = std::process::Command::new(&bin_path)
        .arg("verify")
        .arg(&config_path)
        .current_dir(temp_path)
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&verify_cmd.stdout).unwrap().to_string();
    let stderr = std::str::from_utf8(&verify_cmd.stderr).unwrap().to_string();

    assert!(
        verify_cmd.status.success(),
        "Verify should succeed with matching outputs. Stderr: {stderr}",
    );
    assert!(
        stdout.contains("✓ Verification passed"),
        "Expected success message, got: {stdout}",
    );
}

#[test]
fn test_verify_command_mismatched_outputs() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq file
    let mut input_file = fs::File::create(temp_path.join("input.fq")).unwrap();
    writeln!(input_file, "@read1\nACGT\n+\nIIII").unwrap();

    // Create config
    let config_path = temp_path.join("config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read1 = 'input.fq'

[[step]]
action = 'Head'
n = 1

[output]
prefix = 'output'
"
    )
    .unwrap();

    // Create a fake output file with wrong content
    let mut output_file = fs::File::create(temp_path.join("output_read1.fq")).unwrap();
    writeln!(output_file, "@wrong\nTTTT\n+\nIIII").unwrap();

    // Run verify command - should fail
    let verify_cmd = std::process::Command::new(&bin_path)
        .arg("verify")
        .arg(&config_path)
        .current_dir(temp_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&verify_cmd.stderr).unwrap().to_string();

    assert!(
        !verify_cmd.status.success(),
        "Verify should fail with mismatched outputs"
    );
    assert!(
        stderr.contains("Verification failed") || stderr.contains("mismatch"),
        "Expected error about mismatch, got: {stderr}",
    );
}

#[test]
fn test_verify_command_missing_outputs() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq file
    let mut input_file = fs::File::create(temp_path.join("input.fq")).unwrap();
    writeln!(input_file, "@read1\nACGT\n+\nIIII").unwrap();

    // Create config
    let config_path = temp_path.join("config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read1 = 'input.fq'

[[step]]
action = 'Head'
n = 1

[output]
prefix = 'output'
"
    )
    .unwrap();

    // Don't create any output files - verify should fail

    // Run verify command - should fail due to missing expected outputs
    let verify_cmd = std::process::Command::new(&bin_path)
        .arg("verify")
        .arg(&config_path)
        .current_dir(temp_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&verify_cmd.stderr).unwrap().to_string();

    assert!(
        !verify_cmd.status.success(),
        "Verify should fail with missing outputs"
    );
    assert!(
        stderr.contains("No expected output files found") || stderr.contains("Verification failed"),
        "Expected error about missing files, got: {stderr}",
    );
}

#[test]
fn test_verify_command_auto_detection() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq file
    let mut input_file = fs::File::create(temp_path.join("input.fq")).unwrap();
    writeln!(input_file, "@read1\nACGT\n+\nIIII").unwrap();

    // Create config (single TOML file in directory)
    let config_path = temp_path.join("config.toml");
    let mut config = fs::File::create(&config_path).unwrap();
    writeln!(
        config,
        r"[input]
read1 = 'input.fq'

[[step]]
action = 'Head'
n = 1

[output]
prefix = 'output'
"
    )
    .unwrap();

    // First, generate expected outputs
    let process_cmd = std::process::Command::new(&bin_path)
        .arg("process")
        .current_dir(temp_path)
        .output()
        .unwrap();

    assert!(
        process_cmd.status.success(),
        "Process should succeed with auto-detection"
    );

    // Now verify without specifying config file - should auto-detect
    let verify_cmd = std::process::Command::new(&bin_path)
        .arg("verify")
        .current_dir(temp_path)
        .output()
        .unwrap();

    let stdout = std::str::from_utf8(&verify_cmd.stdout).unwrap().to_string();

    assert!(
        verify_cmd.status.success(),
        "Verify should succeed with auto-detection. Stderr: {}",
        std::str::from_utf8(&verify_cmd.stderr).unwrap()
    );
    assert!(
        stdout.contains("Auto-detected configuration file"),
        "Should show auto-detection message, got: {stdout}",
    );
    assert!(
        stdout.contains("✓ Verification passed"),
        "Should verify successfully, got: {stdout}",
    );
}

#[test]
fn test_verify_command_multiple_toml_files() {
    use std::fs;
    use std::io::Write;

    let current_exe = std::env::current_exe().unwrap();
    let bin_path = current_exe
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("mbf-fastq-processor");

    // Create temp directory
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    // Create test fastq file
    let mut input_file = fs::File::create(temp_path.join("input.fq")).unwrap();
    writeln!(input_file, "@read1\nACGT\n+\nIIII").unwrap();

    // Create two config files
    let config1_path = temp_path.join("config1.toml");
    let mut config1 = fs::File::create(&config1_path).unwrap();
    writeln!(
        config1,
        r"[input]
read1 = 'input.fq'

[output]
prefix = 'output1'
"
    )
    .unwrap();

    let config2_path = temp_path.join("config2.toml");
    let mut config2 = fs::File::create(&config2_path).unwrap();
    writeln!(
        config2,
        r"[input]
read1 = 'input.fq'

[output]
prefix = 'output2'
"
    )
    .unwrap();

    // Try to verify without specifying config file - should fail with multiple files
    let verify_cmd = std::process::Command::new(&bin_path)
        .arg("verify")
        .current_dir(temp_path)
        .output()
        .unwrap();

    let stderr = std::str::from_utf8(&verify_cmd.stderr).unwrap().to_string();

    assert!(
        !verify_cmd.status.success(),
        "Verify should fail when multiple TOML files exist"
    );
    assert!(
        stderr.contains("Found 2 valid TOML files") || stderr.contains("multiple"),
        "Expected error about multiple files, got: {stderr}",
    );
    assert!(
        stderr.contains("Please specify"),
        "Should ask user to specify which file, got: {stderr}",
    );
}
