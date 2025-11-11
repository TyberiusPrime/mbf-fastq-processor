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
