use anyhow::{Context, Result, bail};
use bstr::{BString, ByteSlice};
use ex::fs::{self, DirEntry};
use std::env;
use std::fmt::Write;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use wait_timeout::ChildExt;

#[allow(clippy::missing_panics_doc)]
pub fn run_test(path: &std::path::Path) {
    #[cfg(target_os = "windows")]
    if path.join("skip_windows").exists() {
        println!(
            "Skipping {} on Windows (skip_windows marker present)",
            path.display()
        );
        return;
    }

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
const COMMAND_TIMEOUT: Duration = Duration::from_secs(60); //github might be slow.

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
    let fh = ex::fs::File::open(filename.as_ref())
        .with_context(|| format!("Could not open file {:?}", filename.as_ref()))?;
    let mut wrapped = niffler::send::get_reader(Box::new(fh))?;
    let mut out: Vec<u8> = Vec::new();
    wrapped.0.read_to_end(&mut out)?;
    Ok(std::str::from_utf8(&out)?.to_string())
}

struct TestOutput {
    stdout: BString,
    stderr: BString,
    return_code: i32,
    missing_files: Vec<String>,
    mismatched_files: Vec<(String, String, bool)>,
    unexpected_files: Vec<String>,
}

fn run_command_with_timeout(cmd: &mut std::process::Command) -> Result<std::process::Output> {
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn command")?;

    match child.wait_timeout(COMMAND_TIMEOUT)? {
        Some(status) => {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut reader) = child.stdout.take() {
                reader.read_to_end(&mut stdout)?;
            }
            if let Some(mut reader) = child.stderr.take() {
                reader.read_to_end(&mut stderr)?;
            }
            Ok(std::process::Output {
                status,
                stdout,
                stderr,
            })
        }
        None => {
            let _ = child.kill();
            let status = child.wait()?;
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            if let Some(mut reader) = child.stdout.take() {
                reader.read_to_end(&mut stdout)?;
            }
            if let Some(mut reader) = child.stderr.take() {
                reader.read_to_end(&mut stderr)?;
            }
            let stdout_str = String::from_utf8_lossy(&stdout);
            let stderr_str = String::from_utf8_lossy(&stderr);
            bail!(
                "Command {:?} timed out after {:?}. Exit status: {:?}\nstdout: {}\nstderr: {}",
                &cmd,
                COMMAND_TIMEOUT,
                status,
                stdout_str,
                stderr_str
            );
        }
    }
}

#[cfg(target_os = "windows")]
fn windows_path_to_wsl(path: &Path) -> Result<String> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize {:?}", path))?;
    let mut raw = canonical
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 path: {:?}", canonical))?
        .to_owned();

    if let Some(stripped) = raw.strip_prefix(r"\\?\") {
        raw = stripped.to_owned();
    }

    let mut parts = raw.splitn(2, ':');
    let drive = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing drive letter in path: {raw}"))?;
    let rest = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Missing path component in: {raw}"))?;

    let normalized_rest = rest
        .trim_start_matches(|c| c == '\\' || c == '/')
        .replace('\\', "/");
    let drive_letter = drive
        .chars()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty drive letter in: {raw}"))?
        .to_ascii_lowercase();

    Ok(format!("/mnt/{drive_letter}/{normalized_rest}"))
}

fn run_panic_test(the_test: &TestCase, processor_cmd: &Path) -> Result<()> {
    let rr = perform_test(the_test, processor_cmd)?;
    if rr.return_code == 0 {
        bail!("No panic occurred, but expected one.");
    }
    let expected_panic_file = the_test.dir.join("expected_panic.txt");
    let expected_panic_content: BString = fs::read_to_string(&expected_panic_file)
        .context("Read expected panic file")?
        .trim()
        .into();

    if rr.stderr.find(&expected_panic_content).is_none() {
        anyhow::bail!(
            "{CLI_UNDER_TEST} did not panic in the way that was expected.\nExpected panic: {}\nActual stderr: '{}'",
            expected_panic_content,
            rr.stderr
        );
    }
    if rr.stderr.find(b"FINDME").is_some() {
        anyhow::bail!(
            "{CLI_UNDER_TEST} triggered FINDME\nExpected panic: {}\nActual stderr: '{}'",
            expected_panic_content,
            rr.stderr
        );
    }
    let actual_dir = the_test.dir.join("actual");
    // Create actual directory and copy files
    if actual_dir.exists() {
        fs::remove_dir_all(&actual_dir)?;
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
    for (actual_path, _expected_path, equal_when_ignoring_line_endings) in &rr.mismatched_files {
        if *equal_when_ignoring_line_endings {
            writeln!(
                msg,
                "\t- {actual_path} (mismatched, but equal when ignoring line endings)"
            )
            .unwrap();
        } else {
            writeln!(msg, "\t- {actual_path} (mismatched)").unwrap();
        }
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
        stdout: "".into(),
        stderr: "".into(),
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

    let test_script = test_case.dir.join("test.sh");
    let command_output = if test_script.exists() {
        let script_path = test_script
            .canonicalize()
            .context("Canonicalize test.sh path")?;
        let mut cmd = std::process::Command::new("bash");
        cmd.arg(script_path)
            .env("PROCESSOR_CMD", processor_cmd)
            .env("CONFIG_FILE", &config_file)
            .env("NO_FRIENDLY_PANIC", "1")
            .current_dir(temp_dir.path());
        run_command_with_timeout(&mut cmd).context("Failed to run test.sh")?
    } else {
        let mut cmd = std::process::Command::new(processor_cmd);
        if old_cli_format {
            let old_cli_format_contents = fs::read_to_string(test_case.dir.join("old_cli_format"))
                .context("Read old_cli_format file")?;
            if !old_cli_format_contents.is_empty() {
                cmd.arg(old_cli_format_contents.trim());
            }
        } else {
            cmd.arg("process");
        }
        cmd.arg(&config_file)
            .arg(temp_dir.path())
            .env("NO_FRIENDLY_PANIC", "1")
            .current_dir(temp_dir.path());
        run_command_with_timeout(&mut cmd).context(format!("Failed to run {CLI_UNDER_TEST}"))?
    };

    let stdout = BString::from(command_output.stdout);
    let stderr = BString::from(command_output.stderr);
    result.return_code = command_output.status.code().unwrap_or(-1);
    result.stdout = stdout.clone();
    result.stderr = stderr.clone();

    //for comparison
    fs::write(temp_dir.path().join("stdout"), &stdout).context("Failed to write stdout to file")?;
    fs::write(temp_dir.path().join("stderr"), &stderr).context("Failed to write stderr to file")?;
    /* fs::write(temp_dir.path().join("stderr"), stderr.as_bytes())
    .context("Failed to write stderr to file")?; */
    //for debugging..
    fs::write(actual_dir.as_path().join("stdout"), &stdout)
        .context("Failed to write stdout to file")?;
    fs::write(actual_dir.as_path().join("stderr"), stderr)
        .context("Failed to write stderr to file")?;
    // Check for and run post.sh if it exists
    let post_script = test_case.dir.join("post.sh");
    if post_script.exists() {
        let post_output = run_command_with_timeout(
            std::process::Command::new("bash")
                .arg(post_script.canonicalize().unwrap())
                .current_dir(temp_dir.path()),
        )
        .context("Failed to execute post.sh")?;

        if !post_output.status.success() {
            anyhow::bail!(
                "post.sh failed with exit code: {:?}\nstdout: {}\nstderr: {}",
                post_output.status.code(),
                String::from_utf8_lossy(&post_output.stdout),
                String::from_utf8_lossy(&post_output.stderr)
            );
        }
    }

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
                || file_name_str.starts_with("skip_")
                || file_name_str == "prep.sh"
                || file_name_str == "test.sh"
                || file_name_str == "post.sh"
            {
                return Ok(());
            }
            for only_if_expected_filename in ["stdout", "stderr"] {
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
                                false,
                            ));
                        }
                    } else if expected_path
                        .extension()
                        .is_some_and(|ext| ext == "json" || ext == "html")
                    {
                        //we need to avoid the <working_dir> in reports
                        let actual_content = std::str::from_utf8(&actual_content)
                            .context("Failed to convert actual content to string")?;
                        let working_dir_re =
                            regex::Regex::new(r#""(?P<key>cwd|working_directory)"\s*:\s*"[^"]*""#)
                                .expect("invalid workdir regex");
                        let actual_content = working_dir_re
                            .replace_all(actual_content, |caps: &regex::Captures| {
                                format!("\"{}\": \"WORKINGDIR\"", &caps["key"])
                            })
                            //and the version as well
                            .replace(env!("CARGO_PKG_VERSION"), "X.Y.Z");
                        let actual_content = actual_content.as_bytes().to_vec();
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
                                false,
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
                                false,
                            ));
                        }
                    } else {
                        let expected_content_unified_newline =
                            expected_content.replace("\r\n", "\n");
                        let actual_content_unified_newline = actual_content.replace("\r\n", "\n");
                        let equal_when_ignoring_line_endings =
                            expected_content_unified_newline == actual_content_unified_newline;

                        result.mismatched_files.push((
                            path.to_string_lossy().to_string(),
                            expected_path.to_string_lossy().to_string(),
                            equal_when_ignoring_line_endings,
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
                    || file_name_str.starts_with("skip_")
                    || file_name_str.starts_with("ignore_")
                    || file_name_str == "expected_panic.txt"
                    || file_name_str == "error"
                    || file_name_str == "repeat"
                    || file_name_str == "top.json"
                    || file_name_str == "old_cli_format"
                    || file_name_str == "prep.sh"
                    || file_name_str == "test.sh"
                    || file_name_str == "post.sh"
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
                ex::fs::create_dir_all(dest_path.parent().unwrap())?;

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
        //remove actual dir, since there were no (unexpected) differences
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
    if input_toml_src.exists() {
        //otherwise prep.sh must make it
        fs::copy(&input_toml_src, &input_toml_dst).context("copy input file")?;
    }

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
    // Optional preparatory script
    let prep_script = test_dir.join("prep.sh");
    if prep_script.exists() {
        #[cfg(not(target_os = "windows"))]
        let mut prep_command = {
            let mut command = std::process::Command::new("bash");
            command
                .arg(prep_script.canonicalize().context("canonicalize prep.sh")?)
                .current_dir(temp_dir.path());
            command
        };

        #[cfg(target_os = "windows")]
        let mut prep_command = {
            let script_wsl = windows_path_to_wsl(&prep_script)?;
            let cwd_wsl = windows_path_to_wsl(temp_dir.path())?;
            let mut command = std::process::Command::new("wsl");
            command
                .arg("--cd")
                .arg(&cwd_wsl)
                .arg("bash")
                .arg(&script_wsl);
            command
        };

        let prep_output =
            run_command_with_timeout(&mut prep_command).context("Failed to execute prep.sh")?;

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
