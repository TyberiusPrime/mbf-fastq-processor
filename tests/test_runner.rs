use anyhow::{Context, Result, bail};
use std::fmt::Write;
use ex::fs::{self, DirEntry};
use std::io::Read;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[allow(clippy::missing_panics_doc)]
pub fn run_test(path: &std::path::Path) {
    let panic_file = path.join("expected_panic.txt");
    let mut test_case = TestCase::new(path.to_path_buf());
    let processor_path = find_processor();
    let r = if panic_file.exists() {
        // Run panic test
        test_case.is_panic = true;
        run_panic_test(&test_case, &processor_path)
    } else {
        // Run output test
        run_output_test(&test_case, &processor_path)
    };
    if let Err(e) = r {
        panic!("Test failed {path:?} {e:?}");
    } else {
        println!("Test passed for {}", path.display());
    }
}

const CLI_UNDER_TEST: &str = "mbf-fastq-processor";

fn find_processor() -> PathBuf {
    let exe_path = env!("CARGO_BIN_EXE_mbf-fastq-processor"); //format is not const :(
    PathBuf::from(exe_path)
}

struct TestCase {
    dir: PathBuf,
    is_panic: bool,
}

impl TestCase {
    fn new(dir: PathBuf) -> Self {
        let is_panic = dir.join("expected_panic.txt").exists();
        TestCase { dir, is_panic }
    }
}

fn read_compressed(filename: impl AsRef<Path>) -> Result<String> {
    let fh = std::fs::File::open(filename.as_ref())
        .with_context(|| format!("Could not open file {:?}", filename.as_ref()))?;
    let mut wrapped = niffler::send::get_reader(Box::new(fh))?;
    let mut out: Vec<u8> = Vec::new();
    wrapped.0.read_to_end(&mut out)?;
    Ok(std::str::from_utf8(&out)?.to_string())
}

struct TestOutput {
    stdout: String,
    stderr: String,
    return_code: i32,
    missing_files: Vec<String>,
    mismatched_files: Vec<(String, String)>,
    unexpected_files: Vec<String>,
}

fn run_panic_test(the_test: &TestCase, processor_cmd: &Path) -> Result<()> {
    let rr = perform_test(the_test, processor_cmd)?;
    if rr.return_code == 0 {
        bail!("No panic occurred, but expected one.");
    }
    let expected_panic_file = the_test.dir.join("expected_panic.txt");
    let expected_panic_content = fs::read_to_string(&expected_panic_file)
        .context("Read expected panic file")?
        .trim()
        .to_string();

    if !rr.stderr.contains(&expected_panic_content) {
        anyhow::bail!(
            "{CLI_UNDER_TEST} did not panic as expected.\nExpected panic: {}\nActual stderr: '{}'",
            expected_panic_content,
            rr.stderr
        );
    }
    if rr.stderr.contains("FINDME") {
        anyhow::bail!(
            "{CLI_UNDER_TEST} triggered FINDME\nExpected panic: {}\nActual stderr: '{}'",
            expected_panic_content,
            rr.stderr
        );
    }

    Ok(())
}

fn run_output_test(test_case: &TestCase, processor_cmd: &Path) -> Result<()> {
    let rr = perform_test(test_case, processor_cmd)?;

    if rr.return_code != 0 {
        anyhow::bail!(
            "{CLI_UNDER_TEST} failed with return code: {}\nstdout: {}\nstderr: {}",
            rr.return_code,
            rr.stdout,
            rr.stderr
        );
    }

    let mut msg = String::new();
    for missing_file in &rr.missing_files {
        writeln!(msg, "\t- Expected output file not created: {missing_file}").unwrap();
    }
    for unexpected_file in &rr.unexpected_files {
        writeln!(msg, "\t- Unexpected output file created: {unexpected_file}",).unwrap();
    }
    for (actual_path, _expected_path) in &rr.mismatched_files {
        writeln!(msg, "\t- {actual_path} (mismatched)").unwrap();
    }
    if !msg.is_empty() {
        anyhow::bail!("\toutput files failed verification.\n{msg}");
    }
    Ok(())
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry) -> Result<()>) -> Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry)?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::if_not_else)]
fn perform_test(test_case: &TestCase, processor_cmd: &Path) -> Result<TestOutput> {
    let mut result = TestOutput {
        stdout: String::new(),
        stderr: String::new(),
        return_code: 0,
        missing_files: Vec::new(),
        mismatched_files: Vec::new(),
        unexpected_files: Vec::new(),
    };

    let temp_dir = setup_test_environment(&test_case.dir).context("Setup test dir")?;

    // Run the processor
    let config_file = temp_dir.path().join("input.toml");
    //chdir to temp_dir

    let actual_dir = test_case.dir.join("actual");
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

            // Check if this is a file without read permissions that we can't copy
            let metadata = fs::metadata(&src_path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = metadata.permissions();
                if (perms.mode() & 0o400) == 0 {
                    // No read permission for owner
                    // Create empty file with same permissions instead of copying
                    fs::write(&dest_path, "")?;
                    let mut new_perms = fs::metadata(&dest_path)?.permissions();
                    new_perms.set_mode(perms.mode());
                    fs::set_permissions(&dest_path, new_perms)?;
                    continue;
                }
            }

            fs::copy(&src_path, &dest_path)?;
        }
    }
    let old_cli_format = test_case.dir.join("old_cli_format").exists();

    let mut proc = std::process::Command::new(processor_cmd);

    if old_cli_format {
        let old_cli_format_contents = fs::read_to_string(test_case.dir.join("old_cli_format"))
            .context("Read old_cli_format file")?;
        if !old_cli_format_contents.is_empty() {
            proc.arg(old_cli_format_contents.trim());
        }
    } else {
        proc.arg("process");
    }
    let proc = proc
        .arg(&config_file)
        .arg(temp_dir.path())
        .env("NO_FRIENDLY_PANIC", "1")
        .current_dir(temp_dir.path())
        .output()
        .context(format!("Failed to run {CLI_UNDER_TEST}"))?;

    let stdout = String::from_utf8_lossy(&proc.stdout);
    let stderr = String::from_utf8_lossy(&proc.stderr);
    result.return_code = proc.status.code().unwrap_or(-1);
    result.stdout = stdout.to_string();
    result.stderr = stderr.to_string();

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

    // First, check all files in the temp directory that should match expected outputs
    visit_dirs(temp_dir.path(), &mut |entry: &DirEntry| -> Result<()> {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(temp_dir.path())
            .context("Strip prefix from temp dir path")?;

        // Skip input files and special files
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            let parent_name = path
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_string_lossy();
            if file_name_str.starts_with("input")
                || file_name_str.starts_with("ignore_")
                || parent_name.starts_with("ignore_")
                || file_name_str.starts_with("ignore_")
            {
                return Ok(());
            }
            for only_if_expected_filename in ["stdout"] {
                if file_name_str == only_if_expected_filename {
                    //only check if there's an expected stdout
                    let expected_path = test_case.dir.join(only_if_expected_filename);
                    if !expected_path.exists() {
                        // If there's no expected stdout, skip this file
                        return Ok(());
                    }
                }
            }
        }

        if path.is_file() {
            let expected_path = test_case.dir.join(relative_path);
            if expected_path.exists() {
                // Compare files
                let expected_content = fs::read(&expected_path)?;
                let actual_content = fs::read(&path)?;

                if expected_content != actual_content {
                    //if compressed, compare uncompressed
                    if expected_path
                        .extension()
                        .is_some_and(|ext| ext == "gz" || ext == "zst")
                    {
                        let expected_uncompressed = read_compressed(&expected_path)?;
                        let actual_uncompressed = read_compressed(&path)?;
                        if expected_uncompressed != actual_uncompressed {
                            result.mismatched_files.push((
                                path.to_string_lossy().to_string(),
                                expected_path.to_string_lossy().to_string(),
                            ));
                        }
                    } else if expected_path.extension().is_some_and(|ext| ext == "json") {
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
                                .captures(actual_content)
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
                                actual_content,
                                format!("\"top\": {{ \"_InternalReadCount\": {max_value} }}"),
                            )
                            .as_bytes()
                            .to_vec()
                        } else {
                            actual_content
                        };
                        if actual_content != expected_content {
                            fs::write(&path, &actual_content)
                                .context("Failed to write actual content to file")?;
                            result.mismatched_files.push((
                                path.to_string_lossy().to_string(),
                                expected_path.to_string_lossy().to_string(),
                            ));
                        }
                    } else if expected_path
                        .extension()
                        .is_some_and(|ext| ext == "progress")
                    {
                        //remove all numbres from actual and expected and compare again
                        let expected_wo_numbers = regex::Regex::new(r"\d+")
                            .unwrap()
                            .replace_all(std::str::from_utf8(&expected_content).unwrap(), "");
                        let actual_wo_numbers = regex::Regex::new(r"\d+")
                            .unwrap()
                            .replace_all(std::str::from_utf8(&actual_content).unwrap(), "");
                        if expected_wo_numbers != actual_wo_numbers {
                            result.mismatched_files.push((
                                path.to_string_lossy().to_string(),
                                expected_path.to_string_lossy().to_string(),
                            ));
                        }
                    } else {
                        result.mismatched_files.push((
                            path.to_string_lossy().to_string(),
                            expected_path.to_string_lossy().to_string(),
                        ));
                    }
                }
            } else {
                // Expected file doesn't exist - this is a new output file
                result
                    .unexpected_files
                    .push(path.to_string_lossy().to_string());
            }
        }
        Ok(())
    })?;

    // Also check if there are any expected output files that weren't produced
    for entry in fs::read_dir(&test_case.dir)? {
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
                    || file_name_str == "old_cli_format"
                {
                    continue;
                }

                let actual_path = temp_dir.path().join(file_name);
                if !actual_path.exists() {
                    // Expected output file was not produced
                    result
                        .missing_files
                        .push(expected_path.to_string_lossy().to_string());
                }
            }
        }
    }

    if !(result.missing_files.is_empty()
        && result.mismatched_files.is_empty()
        && result.unexpected_files.is_empty())
    {
        // Create actual directory and copy files
        if actual_dir.exists() {
            fs::remove_dir_all(&actual_dir)?;
        }
        fs::create_dir_all(&actual_dir)?;
        //copy all files from temp_dir to actual_dir
        visit_dirs(temp_dir.path(), &mut |entry| {
            let absolute_src_path = entry.path();
            let relative_src_path = absolute_src_path
                .strip_prefix(temp_dir.path())
                .context("Strip prefix from temp dir path")?;
            if absolute_src_path.is_file() {
                let dest_path = actual_dir.join(relative_src_path);
                std::fs::create_dir_all(dest_path.parent().unwrap())?;

                // Check if this is a file without read permissions that we can't copy
                let metadata = fs::metadata(&absolute_src_path)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = metadata.permissions();
                    if (perms.mode() & 0o400) == 0 {
                        // No read permission for owner
                        // Create empty file with same permissions instead of copying
                        fs::write(&dest_path, "")?;
                        let mut new_perms = fs::metadata(&dest_path)?.permissions();
                        new_perms.set_mode(perms.mode());
                        fs::set_permissions(&dest_path, new_perms)?;
                        return Ok(());
                    }
                }

                fs::copy(&absolute_src_path, &dest_path)?;
            }
            Ok(())
        })?;
    } else {
        //remove actual dir
        if actual_dir.exists() {
            fs::remove_dir_all(&actual_dir)?;
        }
    }
    Ok(result)
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

                    // Check if this is a 0-byte file without read permissions
                    let metadata = fs::metadata(&path)?;
                    if metadata.len() == 0 {
                        // Check if file has no read permissions (using unix permissions)
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let perms = metadata.permissions();
                            if (perms.mode() & 0o400) == 0 {
                                // No read permission for owner
                                // Create empty file without read permissions
                                fs::write(&dst_path, "")?;
                                let mut new_perms = fs::metadata(&dst_path)?.permissions();
                                new_perms.set_mode(perms.mode());
                                fs::set_permissions(&dst_path, new_perms)?;
                                continue;
                            }
                        }
                    }

                    // Normal file copy
                    fs::copy(&path, &dst_path)?;
                }
            }
        }
    }

    // Check for and run prep.sh if it exists
    let prep_script = test_dir.join("prep.sh");
    if prep_script.exists() {
        let prep_output = std::process::Command::new("bash")
            .arg(prep_script.canonicalize().unwrap())
            .current_dir(temp_dir.path())
            .output()
            .context("Failed to execute prep.sh")?;

        if !prep_output.status.success() {
            anyhow::bail!(
                "prep.sh failed with exit code: {:?}\nstdout: {}\nstderr: {}",
                prep_output.status.code(),
                String::from_utf8_lossy(&prep_output.stdout),
                String::from_utf8_lossy(&prep_output.stderr)
            );
        }
    }

    Ok(temp_dir)
}
