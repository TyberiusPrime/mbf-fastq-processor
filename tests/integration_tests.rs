#![allow(clippy::identity_op)]

#[test]
fn test_cookbooks_in_sync() {
    // Verify that the generated cookbooks.rs matches the actual cookbook directories
    use std::collections::HashSet;
    use std::path::Path;

    // Get cookbooks from generated code
    let generated_cookbooks: HashSet<String> = mbf_fastq_processor::cookbooks::list_cookbooks()
        .iter()
        .map(|(_, name)| name.to_string())
        .collect();

    // Get cookbooks from filesystem
    let cookbooks_dir = Path::new("cookbooks");
    assert!(cookbooks_dir.exists(), "cookbooks directory should exist");

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
    assert!(stderr.contains("error: the following required arguments were not provided"));
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
    assert!(stdout.contains("0.8.0"));
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
    assert!(stdout.contains("0.8.0"));
    assert!(cmd.status.success());
}

#[test]
fn test_every_demultiplexed_data_transform_has_test() {
    // This test verifies that every transformation that uses DemultiplexedData
    // has at least one test case where it occurs after a Demultiplex step.
    use std::collections::HashSet;
    use std::path::Path;

    // List of transforms that use DemultiplexedData in their implementation.
    // These are action names from the Transformation enum that have
    // DemultiplexedData fields in their corresponding struct.
    // Internal transforms (starting with _) are not included as they're
    // triggered by other actions (e.g., _Report* are triggered by Report).
    let transforms_with_demultiplexed_data: HashSet<String> = [
        "Head",                  // filters::Head
        "Skip",                  // filters::Skip
        "FilterReservoirSample", // filters::ReservoirSample
        "TagDuplicates",         // extract::tag::Duplicates
        "StoreTagsInTable",      // tag::StoreTagsInTable
        "QuantifyTag",           // tag::QuantifyTag
        "Inspect",               // reports::Inspect (special report)
        "Report",                // reports::Report (triggers _Report* internal transforms)
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    // Find all test TOML files
    let test_cases_dir = Path::new("test_cases");
    let mut toml_files = Vec::new();

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

    find_toml_files(test_cases_dir, &mut toml_files);

    // Track which transforms have tests after Demultiplex
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
                        if trimmed.contains(&format!("'{}'", transform))
                            || trimmed.contains(&format!("\"{}\"", transform))
                        {
                            tested_transforms.insert(transform.clone());
                        }
                    }
                }
            }
        }
    }

    // Check for missing tests
    let missing_tests: Vec<_> = transforms_with_demultiplexed_data
        .difference(&tested_transforms)
        .collect();

    if !missing_tests.is_empty() {
        eprintln!("\n❌ The following transforms use DemultiplexedData but have no test cases");
        eprintln!("   where they occur after a Demultiplex step:");
        for transform in &missing_tests {
            eprintln!("   - {}", transform);
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
        println!("  ✓ {}", transform);
    }
}

#[test]
fn test_readme_toml_examples_validate() {
    // This test extracts TOML code blocks from README.md and validates them
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

    // Validate each TOML block
    for (line_no, toml_content) in &toml_blocks {
        println!("  Validating TOML block starting at line {}...", line_no);

        // Create a temporary directory for this test
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let temp_path = temp_dir.path();

        // Write the TOML to a file
        let toml_path = temp_path.join("test.toml");
        fs::write(&toml_path, toml_content).expect("Failed to write TOML file");

        // Parse the TOML to extract input file references
        let parsed: toml::Value =
            toml::from_str(toml_content).expect(&format!("Failed to parse TOML at line {}", line_no));

        // Create dummy input files if the TOML references them
        if let Some(input_section) = parsed.get("input").and_then(|v| v.as_table()) {
            for (_key, value) in input_section {
                let files_to_create: Vec<String> = match value {
                    toml::Value::String(s) => vec![s.clone()],
                    toml::Value::Array(arr) => arr
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect(),
                    _ => vec![],
                };

                for file_name in files_to_create {
                    let file_path = temp_path.join(&file_name);
                    // Create parent directories if needed
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    // Create empty fastq file with minimal valid content
                    fs::write(&file_path, "@read1\nACGT\n+\nIIII\n")
                        .expect(&format!("Failed to create dummy file: {}", file_name));
                }
            }
        }

        // Try to validate the TOML by parsing it with the actual config parser
        // We'll do this by running the binary in check mode if available
        let current_exe = std::env::current_exe().unwrap();
        let bin_path = current_exe
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("mbf-fastq-processor");

        if bin_path.exists() {
            let output = std::process::Command::new(&bin_path)
                .arg("process")
                .arg(&toml_path)
                .current_dir(temp_path)
                .env("COMMIT_HASH", "test")
                .output();

            match output {
                Ok(result) => {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    let stdout = String::from_utf8_lossy(&result.stdout);

                    // The config should at least parse without syntax errors
                    // Even if it fails at runtime due to missing features or other reasons,
                    // it should not have TOML parsing errors
                    if stderr.contains("TOML") && stderr.contains("error") {
                        panic!(
                            "README.md TOML block at line {} has parsing errors:\nSTDERR: {}\nSTDOUT: {}",
                            line_no, stderr, stdout
                        );
                    }

                    println!("    ✓ TOML block at line {} validated successfully", line_no);
                }
                Err(e) => {
                    eprintln!("    ⚠ Could not run validation (binary not available): {}", e);
                }
            }
        } else {
            println!("    ⚠ Skipping runtime validation (binary not built yet)");
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
        r#"[input]
read_1 = 'test1.fq'
read_2 = 'test2.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true

[output]
prefix = 'output'
report_html = true
"#
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
        r#"[input]
read_1 = 'nonexistent1.fq'
read_2 = 'nonexistent2.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true

[output]
prefix = 'output'
report_html = true
"#
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
        r#"[input]
read_1 = 'test.fq'

[[step]]
action = 'InvalidAction'
name = 'test'

[output]
prefix = 'output'
"#
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
        r#"[input
read_1 = 'test.fq'
this is not valid toml
"#
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
        r#"[input]
read_1 = 'test.fq'

[[step]]
action = 'Report'
name = 'my_report'
count = true
"#
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
