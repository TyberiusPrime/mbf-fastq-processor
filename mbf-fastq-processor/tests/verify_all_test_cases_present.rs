use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
#[allow(clippy::unwrap_used)]
fn all_test_cases_are_generated() {
    let generated = fs::read_to_string("tests/generated.rs").expect("Failed to read generated.rs");

    let mut expected_tests = HashSet::new();
    for search_dir in &[PathBuf::from("../test_cases"), PathBuf::from("../cookbooks")] {
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
            if case_dir.file_name().unwrap() == "actual" {
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

            expected_tests.insert(format!("fn test_cases_x_{name}_{count}()"));
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
fn verify_coobooks_censored() {
    for search_dir in &[PathBuf::from("../test_cases"), PathBuf::from("../cookbooks")] {
        assert!(
            search_dir.exists(),
            "{} directory does not exist",
            search_dir.display()
        );

        let homes_re = regex::Regex::new("/home/([^/]+)").expect("regex wrong");

        for entry in WalkDir::new(search_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_name().to_string_lossy().ends_with(".json")
                    || e.file_name().to_string_lossy().ends_with(".html")
            })
        {
            let content = std::fs::read_to_string(entry.path()).expect("Failed to read file");
            if let Some(hit) = homes_re.captures(&content) {
                if hit.get(1).unwrap().as_str() != "user" {
                    panic!(
                        "found not /home/user home path in {}: {}. Rerun ./dev/censor_cookbooks.py",
                        entry.path().display(),
                        hit.get(0).unwrap().as_str()
                    )
                }
            }
        }
    }
}
