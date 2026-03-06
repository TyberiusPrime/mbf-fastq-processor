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
                violations.push(format!("{}:{}: {}", path.display(), line_no + 1, line.trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found HashMap usage(s) in src/ (use IndexMap instead):\n{}",
        violations.join("\n")
    );
}
