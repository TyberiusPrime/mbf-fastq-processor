use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
#[allow(clippy::unwrap_used)]
fn all_test_cases_are_generated() {
    let generated = fs::read_to_string("tests/generated.rs").expect("Failed to read generated.rs");

    let mut expected_tests = HashSet::new();
    for search_dir in &[
        PathBuf::from("../test_cases"),
        PathBuf::from("../cookbooks"),
    ] {
        assert!(
            search_dir.exists(),
            "{} directory does not exist",
            search_dir.display()
        );

        let mut counters = HashMap::new();

        for entry in WalkDir::new(search_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name().to_string_lossy().starts_with("input")
                    && e.file_name().to_string_lossy().ends_with(".toml")
            })
        {
            let case_dir = entry.path().parent().unwrap().to_owned();
            if case_dir
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("actual")
            {
                continue;
            }
            {
                *counters.entry(case_dir.clone()).or_insert(0) += 1;
            }
            let count = counters.get(&case_dir).expect("just set");

            let name = case_dir
                .strip_prefix(search_dir)
                .unwrap()
                .to_string_lossy()
                .replace(['/', '\\'], "_x_")
                .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_x_")
                .to_lowercase();

            let test_sh = case_dir.join("test.sh");

            if *count > 1 && !test_sh.exists() {
                expected_tests.insert(format!("fn test_cases_x_{name}_{count}()"));
            } else {
                expected_tests.insert(format!("fn test_cases_x_{name}()"));
            }
        }
    }
    for test_fn in expected_tests {
        assert!(
            generated.contains(&test_fn),
            "Missing test function: {test_fn}. Rerun ./dev/update_generated.sh",
        );
    }
}

#[test]
fn verify_all_shell_scripts_pass_shellcheck() {
    let shellcheck = std::process::Command::new("shellcheck")
        .arg("--version")
        .output();

    if shellcheck.is_err() || !shellcheck.expect("shellcheck failure").status.success() {
        panic!("shellcheck not available");
    }

    for search_dir in &[
        PathBuf::from("../test_cases"),
        PathBuf::from("../cookbooks"),
        PathBuf::from("tests"),
    ] {
        if !search_dir.exists() {
            continue;
        }

        for entry in WalkDir::new(search_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with(".sh"))
        {
            let content = std::fs::read_to_string(entry.path()).expect("Failed to read file");

            if content.starts_with("#!/usr/bin/env python3") {
                continue;
            }

            let output = std::process::Command::new("shellcheck")
                .arg(entry.path())
                .output()
                .expect("Failed to run shellcheck");

            if !output.status.success() {
                panic!(
                    "shellcheck failed for {} .\nstdout: {}\nstderr: {}",
                    entry.path().display(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }
}

#[test]
fn verify_coobooks_censored() {
    let homes_re = regex::Regex::new("/home/([^/]+)").expect("regex wrong");
    for search_dir in &[
        PathBuf::from("../test_cases"),
        PathBuf::from("../cookbooks"),
    ] {
        assert!(
            search_dir.exists(),
            "{} directory does not exist",
            search_dir.display()
        );

        for entry in WalkDir::new(search_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name().to_string_lossy().ends_with(".json")
                    || e.file_name().to_string_lossy().ends_with(".html")
            })
        {
            let content = std::fs::read_to_string(entry.path()).expect("Failed to read file");
            if let Some(hit) = homes_re.captures(&content)
                && hit
                    .get(1)
                    .expect("regex can't hit without group 1")
                    .as_str()
                    != "user"
            {
                panic!(
                    "found not /home/user home path in {}: {}. Rerun ./dev/censor_cookbooks.py",
                    entry.path().display(),
                    hit.get(0).expect("Regex hit = group 0 present").as_str()
                )
            }
        }
    }
}
