use regex::Regex;
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

fn extract_section_from_template(template_content: &str, transformation: &str) -> String {
    let action_pattern = format!("# ==== {} ====", transformation);
    let start = template_content.find(&action_pattern).expect(&format!(
        "Could not find section for transformation {transformation} in template.toml",
    ));
    let after_first_newline = template_content[start..]
        .find('\n')
        .map_or(template_content.len(), |pos| start + pos);
    let stop = template_content[after_first_newline..]
        .find("# =")
        .map_or(template_content.len(), |pos| after_first_newline + pos);
    dbg!(&template_content[after_first_newline..stop]);
    template_content[after_first_newline..stop].replace("\n#", "\n")
}

#[test]
fn test_every_step_has_a_template_section() {
    // Get all transformation names from the enum
    let transformations = get_all_transformations();

    // Read the template file
    let template_content =
        fs::read_to_string("src/template.toml").expect("Failed to read template.toml");

    // Check if each transformation is documented in template.toml
    let mut missing = Vec::new();
    for transformation in &transformations {
        let action_pattern = format!("# ==== {} ====", transformation);
        // Skip assertions for transformations not in template - just print a warning
        if !template_content.contains(&action_pattern) {
            missing.push(transformation.clone());
        }
    }
    if !missing.is_empty() {
        missing.sort();
        panic!(
            "Warning: The following transformations are missing in template.toml:\n{}",
            missing.join(", ")
        );
    }

    // Test parsing configuration with each transformation
    for transformation in &transformations {
        let extracted_section = extract_section_from_template(&template_content, transformation);
        if extracted_section.is_empty() {
            panic!("failed to extract section for {transformation} from template.toml");
        }

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
        if extracted_section.contains("label = \"mytag\"") && !extracted_section.contains("= \"Extract")
        {
            //tag based need their tag to not fail the chek
            config.push_str(
                r#"
                [[step]]
                    action = "ExtractRegion"
                    source = "Read1"
                    start = 0
                    length = 3
                    label = "mytag"
            "#,
            );
        }
        config.push_str(&extracted_section);

        // Verify just the parsing
        let mut parsed = toml::from_str::<mbf_fastq_processor::config::Config>(&config)
            .unwrap_or_else(|e| {
                panic!("Could not parse section for {transformation}: {e}.\n{config}",)
            });
        parsed.check().unwrap_or_else(|e| {
            panic!("Error in parsing configuration for {transformation}: {e:?}\n{config}",)
        });
    }
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
                    if !filename.starts_with('_') && filename != "toml.md" && filename != "Options.md" {
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
        .map(|s| s.to_string())
}

fn extract_toml_from_markdown(file_path: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
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
    
    Ok(toml_blocks)
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
            Ok(toml_blocks) => {
                if toml_blocks.is_empty() {
                    failed_files.push(format!("{}: No TOML examples found", doc_file.display()));
                    continue;
                }
                
                for (i, toml_block) in toml_blocks.iter().enumerate() {
                    let request_report = if toml_block.contains("action = \"Report\"") {
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
                    );
                    
                    // Handle tag-based transformations
                    if toml_block.contains("label = ") && !toml_block.contains("= \"Extract") {
                        config.push_str(
                            r#"
[[step]]
    action = "ExtractRegion"
    source = "Read1"
    start = 0
    length = 3
    label = "mytag"

"#,
                        );
                    }
                    
                    config.push_str(toml_block);
                    
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
            Err(e) => {
                failed_files.push(format!("{}: Failed to read file: {}", doc_file.display(), e));
            }
        }
    }
    
    if !failed_files.is_empty() {
        panic!(
            "Documentation TOML validation failed:\n{}",
            failed_files.join("\n")
        );
    }
}
