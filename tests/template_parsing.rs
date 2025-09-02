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
