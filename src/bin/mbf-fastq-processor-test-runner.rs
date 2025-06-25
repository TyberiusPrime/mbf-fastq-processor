use anyhow::{bail, Context, Result};
use std::fs;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use tempfile::TempDir;

fn main() -> Result<()> {
    human_panic::setup_panic!();
    let test_dir = std::env::args().nth(1).unwrap_or("test_cases".to_string());

    run_tests(PathBuf::from(test_dir), false)
}

fn run_tests(test_dir: impl AsRef<Path>, continue_upon_failure: bool) -> Result<()> {
    let last_failed_filename: PathBuf = "/tmp/.mbf-fastq-processor-test-runner-last-failed".into();
    let last_failed: Option<PathBuf> = if last_failed_filename.exists() {
        Some(
            fs::read_to_string(&last_failed_filename)
                .context("Read last failed test case")?
                .trim()
                .into(),
        )
    } else {
        None
    };
    // Find test cases
    let test_dir = test_dir.as_ref();
    let mut test_cases = discover_test_cases(test_dir)?;

    //randomize order
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    test_cases.shuffle(&mut rng);

    if let Some(last_failed) = last_failed {
        //put last failed test to the front - if present
        if test_cases.contains(&last_failed) {
            println!(
                "Found last failed test case: {}. Running it first.",
                last_failed.display()
            );
            test_cases.retain(|x| x != &last_failed);
            test_cases.insert(0, last_failed);
        }
    }

    let mut passed = 0;
    let mut failed = 0;
    let processor_path = find_mbf_fastq_processor()?;

    println!("Found {} test cases", test_cases.len());
    for test_case in test_cases {
        let repeat_count = fs::read_to_string(test_case.join("repeat"))
            .map(|x| {
                x.trim()
                    .parse::<usize>()
                    .expect("Repeat file with non number")
            })
            .unwrap_or(1);

        for repeat in 0..repeat_count {
            print!("\n  Running test: {} {}", test_case.display(), repeat);
            let start = std::time::Instant::now();
            let test_result = if is_panic_test(&test_case)? {
                run_panic_test(&test_case, processor_path.as_ref())
            } else {
                run_output_test(&test_case, processor_path.as_ref())
            };
            let elapsed = start.elapsed();
            print!(" ({}.{:03}s)", elapsed.as_secs(), elapsed.subsec_millis());

            match test_result {
                Ok(()) => {
                    //put checkmark before last line written
                    //so we need minimal lines, but report what we're running
                    print!("\r✅");

                    //println!("✅ Output test passed");
                    passed += 1;
                }
                Err(e) => {
                    //write last failed to file
                    std::fs::write(
                        &last_failed_filename,
                        test_case.to_string_lossy().to_string(),
                    )
                    .ok();
                    print!("\r❌");
                    print!("\n{:?}", e);
                    failed += 1;
                    break; // no more repeats for this one
                }
            }
        }
        if failed > 0 && !continue_upon_failure {
            println!("Stopping due to failure in test: {}", test_case.display());
            break;
        }
    }

    println!("\nTest results: {} passed, {} failed", passed, failed);

    if failed > 0 {
        process::exit(1);
    }

    Ok(())
}
///
/// Finds the full path of a binary in $PATH
fn find_in_path(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH")?
        .to_string_lossy()
        .split(':')
        .map(PathBuf::from)
        .find_map(|dir| {
            let full_path = dir.join(bin);
            if full_path.is_file()
                && fs::metadata(&full_path).ok()?.permissions().mode() & 0o111 != 0
            {
                Some(full_path)
            } else {
                None
            }
        })
}

fn find_mbf_fastq_processor() -> Result<PathBuf> {
    // prefer the one in path
    // if it exists, use that one
    if let Some(path) = find_in_path("mbf_fastq_processor") {
        return Ok(path);
    }
    // otherwise, check if we have a binary next to us
    let current_exe = std::env::current_exe().context("Get current executable path")?;
    let parent = current_exe
        .parent()
        .context("Get parent directory of executable")?;
    if parent.file_name().unwrap().to_string_lossy() == "debug" {
        // run a quick cargo build in debug mod
        std::process::Command::new("cargo")
            .arg("build")
            .status()
            .context("Failed to run cargo build")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("Cargo build failed"))?;
    } else if parent.file_name().unwrap().to_string_lossy() == "release" {
        // run a quick cargo build in release mod
        std::process::Command::new("cargo")
            .arg("build")
            .arg("--release")
            .status()
            .context("Failed to run cargo build")?
            .success()
            .then_some(())
            .ok_or_else(|| anyhow::anyhow!("Cargo build failed"))?;
    }
    let bin_path = current_exe
        .parent()
        .context("Get parent directory of executable")?
        .join("mbf_fastq_processor");

    if !bin_path.exists() {
        anyhow::bail!(
            "mbf_fastq_processor binary not found at: {}",
            bin_path.display()
        );
    }

    Ok(bin_path)
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

fn run_panic_test(test_dir: &Path, processor_cmd: &Path) -> Result<()> {
    let temp_dir = setup_test_environment(test_dir).context("Setup test dir")?;

    // Run the processor
    let config_file = temp_dir.path().join("input.toml");
    //chdir to temp_dir

    let actual_dir = test_dir.join("actual");
    // Create actual directory and copy files
    if actual_dir.exists() {
        fs::remove_dir_all(&actual_dir)?;
    }
    fs::create_dir_all(&actual_dir)?;
    //copy all files from temp_dir to actual_dir
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path.is_file() {
            let dest_path = actual_dir.join(src_path.file_name().unwrap());
            fs::copy(&src_path, &dest_path)?;
        }
    }

    let proc = std::process::Command::new(processor_cmd)
        .arg(&config_file)
        .arg(temp_dir.path())
        .current_dir(temp_dir.path())
        .output()
        .context("Failed to run mbf_fastq_processor")?;

    let stdout = String::from_utf8_lossy(&proc.stdout);
    fs::write(actual_dir.as_path().join("stdout"), stdout.as_bytes())
        .context("Failed to write stdout to file")?;
    let stderr = String::from_utf8_lossy(&proc.stderr);
    fs::write(actual_dir.as_path().join("stderr"), stderr.as_bytes())
        .context("Failed to write stdout to file")?;

    if proc.status.success() {
        let stderr = String::from_utf8_lossy(&proc.stderr);
        anyhow::bail!(
            "mbf_fastq_processor did not fail as expected with status: {}\nstdout: {}\nstderr: {}",
            proc.status,
            stdout,
            stderr
        );
    }

    let expected_panic_file = test_dir.join("expected_panic.txt");
    let expected_panic_content = fs::read_to_string(&expected_panic_file)
        .context("Read expected panic file")?
        .trim()
        .to_string();

    if !stderr.contains(&expected_panic_content) {
        anyhow::bail!(
            "mbf_fastq_processor did not panic as expected.\nExpected panic: {}\nActual stderr: '{}'",
            expected_panic_content,
            stderr
        );
    }

    //remove actual dir
    if actual_dir.exists() {
        fs::remove_dir_all(&actual_dir)?;
    }

    Ok(())
}
enum Error {
    Different,
    Unexpected,
    Missing,
}

fn read_compressed(filename: impl AsRef<Path>) -> Result<String> {
    let fh = std::fs::File::open(filename.as_ref())
        .with_context(|| format!("Could not open file {:?}", filename.as_ref()))?;
    let mut wrapped = niffler::send::get_reader(Box::new(fh))?;
    let mut out: Vec<u8> = Vec::new();
    wrapped.0.read_to_end(&mut out)?;
    Ok(std::str::from_utf8(&out)?.to_string())
}

fn run_output_test(test_dir: &Path, processor_cmd: &Path) -> Result<()> {
    let temp_dir = setup_test_environment(test_dir).context("Setup test dir")?;

    // Run the processor
    let config_file = temp_dir.path().join("input.toml");
    //chdir to temp_dir

    let actual_dir = test_dir.join("actual");
    // Create actual directory and copy files
    if actual_dir.exists() {
        fs::remove_dir_all(&actual_dir)?;
    }
    fs::create_dir_all(&actual_dir)?;
    //copy all files from temp_dir to actual_dir
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let src_path = entry.path();
        if src_path.is_file() {
            let dest_path = actual_dir.join(src_path.file_name().unwrap());
            fs::copy(&src_path, &dest_path)?;
        }
    }

    let proc = std::process::Command::new(processor_cmd)
        .arg(&config_file)
        .arg(temp_dir.path())
        .current_dir(temp_dir.path())
        .output()
        .context("Failed to run mbf_fastq_processor")?;

    let stdout = String::from_utf8_lossy(&proc.stdout);
    let stderr = String::from_utf8_lossy(&proc.stderr);
    //for comparison
    fs::write(temp_dir.path().join("stdout"), stdout.as_bytes())
        .context("Failed to write stdout to file")?;
    /* fs::write(temp_dir.path().join("stderr"), stderr.as_bytes())
        .context("Failed to write stderr to file")?; */
    //for debugging..
    fs::write(actual_dir.as_path().join("stdout"), stdout.as_bytes())
        .context("Failed to write stdout to file")?;
    fs::write(actual_dir.as_path().join("stderr"), stderr.as_bytes())
        .context("Failed to write stderr to file")?;

    if !proc.status.success() {
        anyhow::bail!(
            "mbf_fastq_processor failed with status: {}\nstdout: {}\nstderr: {}",
            proc.status,
            stdout,
            stderr
        );
    }

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
            if file_name_str == "stdout" {
                //only check if there's an expected stdout
                let expected_path = test_dir.join("stdout");
                if !expected_path.exists() {
                    // If there's no expected stdout, skip this file
                    continue;
                }
            }
        }

        if path.is_file() {
            let expected_path = test_dir.join(path.file_name().unwrap());
            if expected_path.exists() {
                // Compare files
                let expected_content = fs::read(&expected_path)?;
                let actual_content = fs::read(&path)?;

                if expected_content != actual_content {
                    //if compressed, compare uncompressed
                    if expected_path
                        .extension()
                        .map_or(false, |ext| ext == "gz" || ext == "zst")
                    {
                        let expected_uncompressed = read_compressed(&expected_path)?;
                        let actual_uncompressed = read_compressed(&path)?;
                        if expected_uncompressed != actual_uncompressed {
                            failures.push((Error::Different, path, expected_path));
                        }
                    } else if expected_path.extension().map_or(false, |ext| ext == "json") {
                        //we need to avoid the <working_dir> in reports
                        let actual_content = std::str::from_utf8(&actual_content)
                            .context("Failed to convert actual content to string")?
                            .replace(temp_dir.path().to_string_lossy().as_ref(), "WORKINGDIR")
                            .as_bytes()
                            .to_vec();
                        //support for _internal_read_count checks.
                        //thease are essentialy <=, but we just want to compare json as strings, bro
                        let irc_top_filename = expected_path.parent().unwrap().join("top.json");
                        let actual_content = if irc_top_filename.exists() {
                            let actual_content = std::str::from_utf8(&actual_content).unwrap();
                            let max_value = serde_json::from_str::<serde_json::Value>(
                                &fs::read_to_string(&irc_top_filename)
                                    .context("Read top.json file")?,
                            )?;
                            let max_value: i64 = max_value.as_i64().unwrap();
                            let re = regex::Regex::new(
                                "\"top\": \\{
    \"_InternalReadCount\": ([0-9]+)
  ",
                            )
                            .unwrap();
                            let hit = re
                                .captures(&actual_content)
                                .and_then(|cap| cap.get(1))
                                .and_then(|m| m.as_str().parse::<i64>().ok())
                                .context(
                                    "top.json present, but no top internal read count found",
                                )?;
                            if hit > max_value {
                                bail!(
                                    "Top internal read count {} exceeds expected maximum {}",
                                    hit,
                                    max_value
                                );
                            }
                            re.replace_all(
                                &actual_content,
                                format!("\"top\": {{ \"_InternalReadCount\": {} }}", max_value),
                            )
                            .as_bytes()
                            .to_vec()
                        } else {
                            actual_content
                        };
                        if actual_content != expected_content {
                            fs::write(&path, &actual_content)
                                .context("Failed to write actual content to file")?;
                            failures.push((Error::Different, path, expected_path));
                        }
                    } else if expected_path
                        .extension()
                        .map_or(false, |ext| ext == "progress")
                    {
                        //remove all numbres from actual and expected and compare again
                        let expected_wo_numbers = regex::Regex::new(r"\d+")
                            .unwrap()
                            .replace_all(std::str::from_utf8(&expected_content).unwrap(), "");
                        let actual_wo_numbers = regex::Regex::new(r"\d+")
                            .unwrap()
                            .replace_all(std::str::from_utf8(&actual_content).unwrap(), "");
                        if expected_wo_numbers != actual_wo_numbers {
                            failures.push((Error::Different, path, expected_path));
                        }
                    } else {
                        failures.push((Error::Different, path, expected_path));
                    }
                }
            } else {
                // Expected file doesn't exist - this is a new output file
                failures.push((Error::Unexpected, path, expected_path));
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
                if file_name_str.starts_with("input")
                    || file_name_str == "expected_panic.txt"
                    || file_name_str == "error"
                    || file_name_str == "repeat"
                    || file_name_str == "top.json"
                {
                    continue;
                }

                let actual_path = temp_dir.path().join(&file_name);
                if !actual_path.exists() {
                    // Expected output file was not produced
                    failures.push((Error::Missing, actual_path, expected_path));
                }
            }
        }
    }

    if !failures.is_empty() {
        // Create actual directory and copy files
        if actual_dir.exists() {
            fs::remove_dir_all(&actual_dir)?;
        }
        fs::create_dir_all(&actual_dir)?;
        //copy all files from temp_dir to actual_dir
        for entry in fs::read_dir(temp_dir.path())? {
            let entry = entry?;
            let src_path = entry.path();
            if src_path.is_file() {
                let dest_path = actual_dir.join(src_path.file_name().unwrap());
                fs::copy(&src_path, &dest_path)?;
            }
        }

        let mut msg = "\tOutput files that don't match expected results:\n".to_string();

        for (err_kind, actual_path, expected_path) in &failures {
            match err_kind {
                Error::Different => {
                    msg.push_str(&format!(
                        "\t- {} (mismatched)\n",
                        actual_path.file_name().unwrap().to_string_lossy()
                    ));
                    msg.push_str(&format!(
                        "\t\t\tdiff '{}' '{}'\n",
                        expected_path.display(),
                        actual_dir
                            .join(expected_path.file_name().unwrap())
                            .display()
                    ));
                }
                Error::Unexpected => msg.push_str(&format!(
                    "\t- Unexpected output file created: {}\n",
                    actual_path.file_name().unwrap().to_string_lossy()
                )),
                Error::Missing => msg.push_str(&format!(
                    "\t- Expected output file not created: {}\n",
                    expected_path.display()
                )),
            }
        }

        anyhow::bail!(
            "\t{} output files failed verification.\n{}",
            failures.len(),
            msg
        );
    }

    //remove actual dir
    if actual_dir.exists() {
        fs::remove_dir_all(&actual_dir)?;
    }
    Ok(())
}

fn setup_test_environment(test_dir: &Path) -> Result<TempDir> {
    let temp_dir = tempfile::tempdir().context("make tempdir")?;

    // Copy input.toml
    let input_toml_src = test_dir.join("input.toml");
    let input_toml_dst = temp_dir.path().join("input.toml");
    fs::copy(&input_toml_src, &input_toml_dst).context("copy input file")?;

    // Copy any input*.fq* files
    for entry in fs::read_dir(test_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(file_name) = path.file_name() {
                let file_name_str = file_name.to_string_lossy();
                if file_name_str.starts_with("input_") {
                    let dst_path = temp_dir.path().join(file_name);
                    fs::copy(&path, &dst_path)?;
                }
            }
        }
    }

    Ok(temp_dir)
}
