use std::path::Path;
use walkdir::WalkDir;

#[test]
fn no_hashmaps_in_src() {
    let mut violations: Vec<String> = Vec::new();

    for entry in WalkDir::new("src")
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

        for (line_no, line) in content.lines().enumerate() {
            if line.contains("HashMap") {
                violations.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    line_no + 1,
                    line.trim()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found HashMap usage(s) in src/ (use IndexMap instead):\n{}",
        violations.join("\n")
    );
}

#[test]
fn symlinks_in_test_cases_are_relative_and_within_repo() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .expect("CARGO_MANIFEST_DIR has no parent");
    let test_cases_dir = repo_root.join("test_cases");
    let canon_repo_root = repo_root
        .canonicalize()
        .expect("Failed to canonicalize repo root");

    let mut violations: Vec<String> = Vec::new();

    for entry in WalkDir::new(&test_cases_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path_is_symlink())
    {
        let link_path = entry.path();

        // Skip symlinks inside actual/ dirs — those are created at test runtime by verify
        if link_path.components().any(|c| c.as_os_str() == "actual") {
            continue;
        }

        let target = match std::fs::read_link(link_path) {
            Ok(t) => t,
            Err(e) => {
                violations.push(format!(
                    "{}: failed to read symlink: {e}",
                    link_path.display()
                ));
                continue;
            }
        };

        if target.is_absolute() {
            violations.push(format!(
                "{} -> {} (absolute path; must be relative)",
                link_path.display(),
                target.display()
            ));
            continue;
        }

        // Resolve the target relative to the symlink's parent directory
        let resolved = link_path
            .parent()
            .expect("symlink has no parent")
            .join(&target);
        match resolved.canonicalize() {
            Ok(canon) if !canon.starts_with(&canon_repo_root) => {
                violations.push(format!(
                    "{} -> {} (resolves outside the repo to {})",
                    link_path.display(),
                    target.display(),
                    canon.display()
                ));
            }
            Err(e) => {
                violations.push(format!(
                    "{} -> {} (broken: {e})",
                    link_path.display(),
                    target.display()
                ));
            }
            Ok(_) => {}
        }
    }

    assert!(
        violations.is_empty(),
        "Found symlink violation(s) in test_cases/ (all symlinks must be relative and resolve within the repo):\n{}",
        violations.join("\n")
    );
}
