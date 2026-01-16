use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[test]
#[allow(clippy::unwrap_used)]
fn all_test_cases_are_generated() {
    let generated = fs::read_to_string("tests/generated.rs").expect("Failed to read generated.rs");

    let mut expected_tests = HashSet::new();
    let search_dir = PathBuf::from("../test_cases");
    assert!(search_dir.exists(), "test_cases directory does not exist");

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
            .strip_prefix("../test_cases")
            .unwrap()
            .to_string_lossy()
            .replace(['/', '\\'], "_x_")
            .replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_x_")
            .to_lowercase();

        expected_tests.insert(format!("fn test_cases_x_{name}_{count}()"));
    }

    for test_fn in expected_tests {
        assert!(
            generated.contains(&test_fn),
            "Missing test function: {test_fn}. Rerun ./dev/update_generated.sh",
        );
    }
}
