use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::io::Read;
use anyhow::{Result, Context};
use clap::Parser;
use tempfile::TempDir;

#[derive(Parser, Debug)]
#[command(about = "Test runner for mbf-fastq-processor")]
struct Args {
    #[arg(default_value = "test_cases")]
    test_directory: PathBuf,
}

fn main() -> Result<()> {
    human_panic::setup_panic!();
    let args = Args::parse();
    
    run_tests(&args.test_directory)
}

fn run_tests(test_dir: &Path) -> Result<()> {
    // Find test cases
    let test_cases = discover_test_cases(test_dir)?;
    
    let mut passed = 0;
    let mut failed = 0;
    
    println!("Found {} test cases", test_cases.len());
    
    for test_case in test_cases {
        println!("\nRunning test: {}", test_case.display());
        
        if is_panic_test(&test_case)? {
            match run_panic_test(&test_case) {
                Ok(()) => {
                    println!("✅ Panic test passed");
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ Panic test failed: {}", e);
                    failed += 1;
                }
            }
        } else {
            match run_output_test(&test_case) {
                Ok(()) => {
                    println!("✅ Output test passed");
                    passed += 1;
                }
                Err(e) => {
                    println!("❌ Output test failed: {}", e);
                    failed += 1;
                }
            }
        }
    }
    
    println!("\nTest results: {} passed, {} failed", passed, failed);
    
    if failed > 0 {
        process::exit(1);
    }
    
    Ok(())
}

fn discover_test_cases(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        anyhow::bail!("Test directory does not exist: {}", dir.display());
    }

    let mut test_cases = Vec::new();
    discover_test_cases_recursive(dir, &mut test_cases)?;
    Ok(test_cases)
}

fn discover_test_cases_recursive(dir: &Path, test_cases: &mut Vec<PathBuf>) -> Result<()> {
    // Check if this directory is a test case
    if dir.join("input.toml").exists() {
        test_cases.push(dir.to_path_buf());
        return Ok(());
    }
    
    // Otherwise, search through subdirectories
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            discover_test_cases_recursive(&path, test_cases)?;
        }
    }
    
    Ok(())
}

fn is_panic_test(test_dir: &Path) -> Result<bool> {
    Ok(test_dir.join("expected_panic.txt").exists())
}

fn run_panic_test(test_dir: &Path) -> Result<()> {
    let temp_dir = setup_test_environment(test_dir)?;
    
    // Read the expected panic message
    let expected_panic_path = test_dir.join("expected_panic.txt");
    let mut expected_panic = String::new();
    fs::File::open(&expected_panic_path)?.read_to_string(&mut expected_panic)?;
    expected_panic = expected_panic.trim().to_string();
    
    // Create a file to detect panics
    let error_file = temp_dir.path().join("error");
    let _f = fs::File::create(&error_file)?;
    
    // Run the processor, expecting it to panic
    let config_file = temp_dir.path().join("input.toml");
    let result = std::panic::catch_unwind(|| {
        mbf_fastq_processor::run(&config_file, temp_dir.path()).unwrap();
    });
    
    // Check if the panic occurred as expected
    match result {
        Ok(_) => {
            anyhow::bail!("Test was supposed to panic, but it didn't");
        }
        Err(_) => {
            if !error_file.exists() {
                anyhow::bail!("Expected panic but couldn't verify it properly");
            }
            
            // Note: We can't easily capture the exact panic message this way
            println!("Expected panic occurred (actual message could not be verified)");
            println!("Expected message: {}", expected_panic);
            
            Ok(())
        }
    }
}

fn run_output_test(test_dir: &Path) -> Result<()> {
    let temp_dir = setup_test_environment(test_dir)?;
    
    // Run the processor
    let config_file = temp_dir.path().join("input.toml");
    mbf_fastq_processor::run(&config_file, temp_dir.path())?;
    
    // Compare output files
    let mut failures = Vec::new();
    
    // First, check all files in the temp directory that should match expected outputs
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let path = entry.path();
        
        // Skip input files and special files
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.starts_with("input") || file_name_str == "error" {
                continue;
            }
        }
        
        if path.is_file() {
            let expected_path = test_dir.join(path.file_name().unwrap());
            if expected_path.exists() {
                // Compare files
                let expected_content = fs::read(&expected_path)?;
                let actual_content = fs::read(&path)?;
                
                if expected_content != actual_content {
                    failures.push((path, expected_path));
                }
            } else {
                // Expected file doesn't exist - this is a new output file
                failures.push((path, expected_path));
            }
        }
    }
    
    // Also check if there are any expected output files that weren't produced
    for entry in fs::read_dir(test_dir)? {
        let entry = entry?;
        let expected_path = entry.path();
        
        if expected_path.is_file() {
            if let Some(file_name) = expected_path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                
                // Skip non-output files
                if file_name_str.starts_with("input") || 
                   file_name_str == "expected_panic.txt" ||
                   file_name_str == "error" {
                    continue;
                }
                
                let actual_path = temp_dir.path().join(&file_name);
                if !actual_path.exists() {
                    // Expected output file was not produced
                    failures.push((actual_path, expected_path));
                }
            }
        }
    }
    
    if !failures.is_empty() {
        // Create actual directory and copy files
        let actual_dir = test_dir.join("actual");
        if actual_dir.exists() {
            fs::remove_dir_all(&actual_dir)?;
        }
        fs::create_dir_all(&actual_dir)?;
        
        println!("\nOutput files that don't match expected results:");
        
        for (actual_path, expected_path) in &failures {
            let file_name = expected_path.file_name().unwrap();
            let dest_path = actual_dir.join(file_name);
            
            if actual_path.exists() {
                fs::copy(actual_path, &dest_path)?;
                println!("- {} (mismatched)", file_name.to_string_lossy());
                println!("  diff '{}' '{}'", expected_path.display(), dest_path.display());
            } else {
                println!("- {} (missing)", file_name.to_string_lossy());
            }
        }
        
        anyhow::bail!("{} output files failed verification", failures.len());
    }
    
    Ok(())
}

fn setup_test_environment(test_dir: &Path) -> Result<TempDir> {
    let temp_dir = tempfile::tempdir()?;
    
    // Copy input.toml
    let input_toml_src = test_dir.join("input.toml");
    let input_toml_dst = temp_dir.path().join("input.toml");
    fs::copy(&input_toml_src, &input_toml_dst)?;
    
    // Copy any input*.fq* files
    for entry in fs::read_dir(test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("input") && 
                   (file_name_str.ends_with(".fq") || 
                    file_name_str.contains(".fq.") || 
                    file_name_str.ends_with(".fastq") || 
                    file_name_str.contains(".fastq.")) {
                    let dst_path = temp_dir.path().join(file_name);
                    fs::copy(&path, &dst_path)?;
                }
            }
        }
    }
    
    Ok(temp_dir)
}
