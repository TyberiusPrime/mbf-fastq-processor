///eserde has a bug that it will list skipped fields as missing if a required field is missing
///this verifies we have set default on all of them.
use std::fs;
use std::path::{Path, PathBuf};
fn scan_directory(dir: &Path, files: &mut Vec<PathBuf>) {
    if dir.exists() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            } else if path.is_dir() {
                scan_directory(&path, files);
            }
        }
    }
}

fn get_all_transformation_files() -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Add the main transformations.rs file
    files.push(PathBuf::from("src/transformations.rs"));

    // Recursively scan the transformations directory
    scan_directory(Path::new("src/transformations"), &mut files);

    files
}

fn check_serde_skip_preceded_by_default(file_path: &Path) -> Result<(), Vec<String>> {
    let content = fs::read_to_string(file_path).map_err(|e| {
        vec![format!(
            "Failed to read file {}: {}",
            file_path.display(),
            e
        )]
    })?;

    let lines: Vec<&str> = content.lines().collect();
    let mut errors = Vec::new();

    for (line_idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        // Check if this line contains #[serde(skip)]
        if trimmed.contains("#[serde(skip)]") && !trimmed.contains("// nodefault") {
            let line_number = line_idx + 1; // Convert to 1-based line numbering

            // Look for the preceding #[serde(default)] line
            let mut found_default = false;

            // Search backwards from the current line
            for prev_idx in (0..line_idx).rev() {
                let prev_line = lines[prev_idx].trim();

                // If we hit a non-attribute line that's not just whitespace or comments, stop searching
                if !prev_line.is_empty()
                    && !prev_line.starts_with('#')
                    && !prev_line.starts_with("//")
                {
                    break;
                }

                // Check if this line contains #[serde(default)]
                if prev_line.contains("#[serde(default)]") {
                    found_default = true;
                    break;
                }
            }

            if !found_default {
                errors.push(format!(
                    "{}:{}: Found #[serde(skip)] without preceding #[serde(default)]",
                    file_path.display(),
                    line_number
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[test]
fn test_serde_skip_preceded_by_default() {
    let files = get_all_transformation_files();
    let mut all_errors = Vec::new();

    for file_path in files {
        if let Err(mut errors) = check_serde_skip_preceded_by_default(&file_path) {
            all_errors.append(&mut errors);
        }
    }

    assert!(
        all_errors.is_empty(),
        "Found serde attribute violations:\n{}",
        all_errors.join("\n")
    );
}
