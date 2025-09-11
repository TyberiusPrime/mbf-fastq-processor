use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn get_all_transformations() -> Vec<String> {
    let transformations_content =
        fs::read_to_string("src/transformations.rs").expect("Failed to read transformations.rs");

    // Find the Transformation enum
    let enum_start = transformations_content
        .find("pub enum Transformation {")
        .expect("Could not find 'pub enum Transformation {' in transformations.rs");

    // Find the matching closing brace
    let content_after_enum = &transformations_content[enum_start..];
    let enum_end = content_after_enum
        .find("\n}\n")
        .expect("Could not find opening brace for enum");

    let enum_content = &content_after_enum[..enum_end];

    // Extract transformation names using regex
    let re = Regex::new(r"^\s*([A-Z][A-Za-z0-9_]*)\s*[\(,]").unwrap();
    let mut transformations = Vec::new();

    for line in enum_content.lines() {
        if let Some(captures) = re.captures(line) {
            if let Some(name) = captures.get(1) {
                let transformation_name = name.as_str().to_string();
                // Skip internal transformations (those starting with underscore)
                if !transformation_name.starts_with('_') {
                    transformations.push(transformation_name);
                }
            }
        }
    }

    transformations
}

fn get_transformation_target_patterns() -> HashMap<String, &'static str> {
    let mut patterns = HashMap::new();

    // Dynamically discover all Rust files in src/transformations/
    let transformations_dir = Path::new("src/transformations");
    if let Ok(entries) = fs::read_dir(transformations_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(&path) {
                    analyze_transformations_in_file(&content, &mut patterns);
                }
            }
        }
    }

    // Handle deprecated transformations that have target fields but are deprecated
    // These should be excluded from pattern checking
    patterns.remove("TrimAdapterMismatchTail");
    patterns.remove("TrimPolyTail");

    patterns
}

fn analyze_transformations_in_file(content: &str, patterns: &mut HashMap<String, &'static str>) {
    // Use regex to find all struct definitions with their content
    let struct_regex = Regex::new(r"(?s)pub struct (\w+)\s*\{([^}]+)\}").unwrap();

    for captures in struct_regex.captures_iter(content) {
        let struct_name = captures.get(1).unwrap().as_str();
        let struct_body = captures.get(2).unwrap().as_str();

        // Look for segment field in the struct body (must be "pub segment:" to avoid false matches)
        if struct_body.contains("pub segment:") {
            if struct_body.contains("SegmentIndexOrAll") {
                patterns.insert(
                    struct_name.to_string(),
                    r#"target = "read1" # Any of your input segments, or 'All'"#,
                );
            } else if struct_body.contains("SegmentIndex") {
                patterns.insert(
                    struct_name.to_string(),
                    r#"target = "read1" # Any of your input segments"#,
                );
            }
        }

        // Check for source field (special case for ExtractRegion)
        if struct_body.contains("pub segment:") && struct_body.contains("SegmentIndex") {
            patterns.insert(
                struct_name.to_string(),
                r#"target = "read1" # Any of your input segments"#,
            );
        }
    }
}

fn check_target_pattern_in_text(text: &str, transformation: &str, expected_pattern: &str) -> bool {
    // Check for target patterns - simplified version
    if expected_pattern.contains("Any of your input segments, or 'All'") {
        // Should contain "All" in the comment
        return text.contains("Any of your input segments, or 'All'");
    } else if expected_pattern.contains("Any of your input segments") {
        // Should contain the 4 base targets but not "All"
        return text.contains("Any of your input segments")
            && !text.contains("Any of your input segments, or 'All'");
    }

    // Handle special case for ExtractRegion which uses "source" instead of "target"
    if transformation == "ExtractRegion" {
        return text.contains("segment") && text.contains("Any of your input segments");
    }

    true // Skip transformations without target fields
}

fn extract_section_from_template(template_content: &str, transformation: &str) -> String {
    let action_pattern = format!("# ==== {transformation} ====");
    let start = template_content.find(&action_pattern).unwrap_or_else(|| {
        panic!("Could not find section for transformation {transformation} in template.toml",)
    });
    let after_first_newline = template_content[start..]
        .find('\n')
        .map_or(template_content.len(), |pos| start + pos);
    let stop = template_content[after_first_newline..]
        .find("# =")
        .map_or(template_content.len(), |pos| after_first_newline + pos);
    template_content[after_first_newline..stop].replace("\n#", "\n")
}

fn prep_config_to_parse(extracted_section: &str) -> String {
    let request_report = if extracted_section.contains("action = \"Report\"") {
        "true"
    } else {
        "false"
    };

    let mut config = format!(
        r#"
[input]
read1 = "test_r1.fastq"
read2 = "test_r2.fastq"

[output]
prefix = "output"
format = "raw"
report_json = {request_report}
report_html = false

"#
    )
    .to_string();

    // Add appropriate tag extraction steps based on what the transformation needs
    if (extracted_section.contains("label = ")
        || extracted_section.contains("action = \"StoreTagsInTable\""))
        && !extracted_section.contains("= \"Extract")
    {
        // Check what type of filter this is and add the appropriate tag extraction step
        if extracted_section.contains("action = \"FilterByNumericTag\"") {
            //numeric filters need numeric tags
            config.push_str(
                r#"
                [[step]]
                    action = "ExtractLength"
                    segment = "read1"
                    label = "mytag"
            "#,
            );
        } else if extracted_section.contains("action = \"FilterByBoolTag\"") {
            //bool filters need bool tags
            config.push_str(
                r#"
                [[step]]
                    action = "TagDuplicates"
                    segment = "read1"
                    label = "mytag"
                    false_positive_rate = 0.0
                    seed = 42
            "#,
            );
        } else {
            //default to location tags for other filters like FilterByTag
            config.push_str(
                r#"
                [[step]]
                    action = "ExtractRegion"
                    segment = "read1"
                    start = 0
                    length = 3
                    label = "mytag"
            "#,
            );
        }
    }
    config.push_str(extracted_section);
    config
}

#[test]
fn test_every_step_has_a_template_section() {
    // Get all transformation names from the enum
    let transformations = get_all_transformations();

    // Read the template file
    let template_content =
        fs::read_to_string("src/template.toml").expect("Failed to read template.toml");

    let mut errors = Vec::new();

    // Check if each transformation is documented in template.toml
    let mut missing = Vec::new();
    for transformation in &transformations {
        let action_pattern = format!("# ==== {transformation} ====");
        // Skip assertions for transformations not in template - just collect them
        if !template_content.contains(&action_pattern) {
            missing.push(transformation.clone());
        }
    }
    if !missing.is_empty() {
        missing.sort();
        errors.push(format!(
            "The following transformations are missing in template.toml:\n{}",
            missing.join(", ")
        ));
    }

    // Test parsing configuration with each transformation and check target patterns
    let target_patterns = get_transformation_target_patterns();

    for transformation in &transformations {
        // Skip if transformation is missing from template
        if missing.contains(transformation) {
            continue;
        }
        let extracted_section =
            match extract_section_from_template(&template_content, transformation) {
                section if section.is_empty() => {
                    errors.push(format!(
                        "Failed to extract section for {transformation} from template.toml"
                    ));
                    continue;
                }
                section => section,
            };

        // Check target pattern consistency if transformation has a target field
        // Skip deprecated transformations
        if extracted_section.contains("deprecated") {
            // Skip pattern checking for deprecated transformations
        } else if let Some(expected_pattern) = target_patterns.get(transformation) {
            if !check_target_pattern_in_text(&extracted_section, transformation, expected_pattern) {
                errors.push(format!(
                    "Template section for {transformation} does not contain the correct target pattern.\nExpected pattern like: {expected_pattern}\nActual section:\n{extracted_section}"
                ));
            }
        }

        let config = prep_config_to_parse(&extracted_section);

        // Verify just the parsing
        match toml::from_str::<mbf_fastq_processor::config::Config>(&config) {
            Ok(mut parsed) => {
                if let Err(e) = parsed.check() {
                    errors.push(format!(
                        "Error in parsing configuration for {transformation}: {e:?}\n{config}"
                    ));
                }
            }
            Err(e) => {
                errors.push(format!(
                    "Could not parse section for {transformation}: {e}.\n{config}"
                ));
            }
        }
    }

    assert!(
        errors.is_empty(),
        "Template validation failed:\n{}",
        errors.join("\n")
    );
}

fn get_all_doc_files() -> Vec<PathBuf> {
    let mut doc_files = Vec::new();
    let docs_dir = Path::new("docs/content/docs/reference");

    if docs_dir.exists() {
        visit_dir_recursive(docs_dir, &mut doc_files);
    }

    doc_files
}

fn visit_dir_recursive(dir: &Path, doc_files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit_dir_recursive(&path, doc_files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
                // Skip index files and general documentation
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if !filename.starts_with('_')
                        && filename != "toml.md"
                        && filename != "Options.md"
                    {
                        doc_files.push(path);
                    }
                }
            }
        }
    }
}

fn extract_transformation_from_filename(file_path: &Path) -> Option<String> {
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}

fn extract_toml_from_markdown(
    file_path: &Path,
) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    if content.contains("not-a-transformation: true") {
        return Ok(None);
    }

    let mut toml_blocks = Vec::new();
    let mut in_toml_block = false;
    let mut current_block = Vec::new();

    for line in content.lines() {
        if line.trim() == "```toml" {
            in_toml_block = true;
            current_block.clear();
        } else if line.trim() == "```" && in_toml_block {
            in_toml_block = false;
            if !current_block.is_empty() {
                toml_blocks.push(current_block.join("\n"));
            }
        } else if in_toml_block {
            current_block.push(line);
        }
    }

    Ok(Some(toml_blocks))
}

#[test]
fn test_every_transformation_has_documentation() {
    let transformations = get_all_transformations();
    let doc_files = get_all_doc_files();

    // Create a set of documented transformations
    let mut documented_transformations = std::collections::HashSet::new();
    for doc_file in &doc_files {
        if let Some(transformation) = extract_transformation_from_filename(doc_file) {
            documented_transformations.insert(transformation);
        }
    }

    // Check for missing documentation
    let mut missing_docs = Vec::new();
    for transformation in &transformations {
        if !documented_transformations.contains(transformation) {
            missing_docs.push(transformation.clone());
        }
    }

    if !missing_docs.is_empty() {
        missing_docs.sort();
        panic!(
            "The following transformations are missing documentation files:\n{}",
            missing_docs.join(", ")
        );
    }
}

#[test]
fn test_documentation_toml_examples_parse() {
    let doc_files = get_all_doc_files();
    let mut failed_files = Vec::new();

    for doc_file in &doc_files {
        let transformation = extract_transformation_from_filename(doc_file).unwrap();

        match extract_toml_from_markdown(doc_file) {
            Ok(Some(toml_blocks)) => {
                if toml_blocks.is_empty() {
                    failed_files.push(format!("{}: No TOML examples found", doc_file.display()));
                    continue;
                }

                let target_patterns = get_transformation_target_patterns();

                for (i, toml_block) in toml_blocks.iter().enumerate() {
                    if !toml_block.contains(&format!("action = \"{transformation}\"")) {
                        failed_files.push(format!(
                            "{}: TOML block {} does not contain action = \"{transformation}\"",
                            doc_file.display(),
                            i + 1,
                        ));
                        continue;
                    }

                    // Check target pattern consistency if transformation has a target field
                    if let Some(expected_pattern) = target_patterns.get(&transformation) {
                        if !check_target_pattern_in_text(
                            toml_block,
                            &transformation,
                            expected_pattern,
                        ) {
                            failed_files.push(format!(
                                "{}: TOML block {} does not contain the correct target pattern.\nExpected pattern like: {}\nActual block:\n{}",
                                doc_file.display(),
                                i + 1,
                                expected_pattern,
                                toml_block
                            ));
                        }
                    }

                    let config = prep_config_to_parse(toml_block);

                    // Try to parse the configuration
                    match toml::from_str::<mbf_fastq_processor::config::Config>(&config) {
                        Ok(mut parsed_config) => {
                            if let Err(e) = parsed_config.check() {
                                failed_files.push(format!(
                                    "{}: TOML block {} failed validation: {:?}",
                                    doc_file.display(),
                                    i + 1,
                                    e
                                ));
                            }
                        }
                        Err(e) => {
                            failed_files.push(format!(
                                "{}: TOML block {} failed to parse: {}",
                                doc_file.display(),
                                i + 1,
                                e
                            ));
                        }
                    }
                }
            }
            Ok(None) => {
                // had not-a-transformation: true
            }
            Err(e) => {
                failed_files.push(format!(
                    "{}: Failed to read file: {}",
                    doc_file.display(),
                    e
                ));
            }
        }
    }

    assert!(
        failed_files.is_empty(),
        "Documentation TOML validation failed:\n{}",
        failed_files.join("\n")
    );
}
