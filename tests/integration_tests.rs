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
