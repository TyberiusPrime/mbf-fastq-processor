use regex::Regex;
use std::fs;

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
    let lines: Vec<&str> = template_content.lines().collect();
    let mut result = String::new();
    let mut in_section = false;
    let mut step_started = false;

    let action_pattern = format!("action = \"{}\"", transformation);

    for (i, line) in lines.iter().enumerate() {
        if line.contains(&action_pattern) {
            in_section = true;
            step_started = true;
            // Find the start of this [[step]] section
            for j in (0..=i).rev() {
                if lines[j].trim().starts_with("[[step") {
                    result.push_str(lines[j]);
                    result.push('\n');
                    break;
                }
            }
            if !result.contains("[[step") {
                result.push_str("[[step]]\n");
            }
            result.push_str(line);
            result.push('\n');
        } else if in_section && step_started {
            if line.trim().starts_with("[[") || line.trim().starts_with("[") {
                // End of current section
                break;
            }
            if line.trim().starts_with("#") {
                continue; // Skip comments within the section
            }

            // Fix template entries that have comments instead of values
            let mut processed_line = line.to_string();

            // Handle various template formatting issues
            if processed_line.contains(" = #")
                || processed_line.contains(" = int")
                || processed_line.contains(" = positive")
                || processed_line.contains(" = u8")
                || processed_line.contains(" = bool")
                || processed_line.contains(" = float")
                || processed_line.contains("Read1|Read2")
            {
                // Replace with a default value based on the field name
                if processed_line.contains("n =") || processed_line.contains("length =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = 1";
                } else if processed_line.contains("target =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"Read1\"";
                } else if processed_line.contains("min =") || processed_line.contains("quality =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = 33";
                } else if processed_line.contains("seed =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = 42";
                } else if processed_line.contains("false_positive_rate =")
                    || processed_line.contains("p =")
                {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = 0.01";
                } else if processed_line.contains("invert =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = false";
                } else if processed_line.contains("anchor =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"Left\"";
                } else if processed_line.contains("query =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = \"ATG\"";
                } else if processed_line.contains("label =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"test\"";
                } else if processed_line.contains("keep_or_remove =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"Keep\"";
                } else if processed_line.contains("filename =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"test.fq\"";
                } else if processed_line.contains("adapter =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"AGATCGGAAG\"";
                } else if processed_line.contains("base =") {
                    processed_line =
                        processed_line.split(" = ").next().unwrap_or("").to_string() + " = \"A\"";
                } else if processed_line.contains("allowed =") {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"ATGC\"";
                } else {
                    processed_line = processed_line.split(" = ").next().unwrap_or("").to_string()
                        + " = \"placeholder\"";
                }
            }

            if !processed_line.trim().is_empty() {
                result.push_str(&processed_line);
                result.push('\n');
            }
        }
    }

    result
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
        let action_pattern = format!("==== {} ====", transformation);
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
            continue; // Skip transformations without examples in template
        }

        let mut config = r#"
[input]
read1 = "test_r1.fastq"
read2 = "test_r2.fastq"

[output]
prefix = "output"
format = "raw"
report_json = true
report_html = false

"#
        .to_string();
        config.push_str(&extracted_section);

        // Verify just the parsing
        let mut parsed = toml::from_str::<mbf_fastq_processor::config::Config>(&config)
            .unwrap_or_else(|e| panic!("Could not parse section for {}: {}", transformation, e));
        parsed
            .check()
            .unwrap_or_else(|e| panic!("Error in configuration for {}: {}", transformation, e));
    }
}
