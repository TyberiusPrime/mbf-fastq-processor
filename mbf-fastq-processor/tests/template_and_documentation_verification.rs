use regex::Regex;
use schemars::schema_for;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use tempfile::tempdir;

static STRUCT_REGEX: OnceLock<Regex> = OnceLock::new();
static TRANSFORMATION_SCHEMA: OnceLock<serde_json::Value> = OnceLock::new();

/// Get all transformation names from the JSON schema
fn get_all_transformations() -> Vec<String> {
    let schema = get_transformation_schema();
    let mut transformations = Vec::new();

    // Navigate to the oneOf array in the schema
    let one_ofs = schema
        .get("oneOf")
        .and_then(|o| o.as_array())
        .expect("Schema does not contain oneOf array");

    for variant in one_ofs {
        if let Some(action_const) = variant
            .get("properties")
            .and_then(|p| p.get("action"))
            .and_then(|a| a.get("const"))
            .and_then(|c| c.as_str())
        {
            // Skip internal transformations (those starting with underscore)
            if !action_const.starts_with('_') {
                transformations.push(action_const.to_string());
            }
        }
    }

    transformations.sort();
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
    /* patterns.remove("TrimAdapterMismatchTail");
    patterns.remove("TrimPolyTail"); */

    patterns
}

fn analyze_transformations_in_file(content: &str, patterns: &mut HashMap<String, &'static str>) {
    // Use regex to find all struct definitions with their content
    let struct_regex =
        STRUCT_REGEX.get_or_init(|| Regex::new(r"(?s)pub struct (\w+)\s*\{([^}]+)\}").unwrap());

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

fn collect_actions(section: &str) -> Vec<String> {
    section
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("action") {
                if let Some(first_quote) = rest.find('"') {
                    let remaining = &rest[first_quote + 1..];
                    if let Some(end_quote) = remaining.find('"') {
                        return Some(remaining[..end_quote].to_string());
                    }
                }
            }
            None
        })
        .collect()
}

fn get_template_section_names(template_content: &str) -> Vec<(String, usize)> {
    template_content
        .lines()
        .enumerate()
        .filter_map(|(line_no, line)| {
            let trimmed = line.trim();
            let without_prefix = trimmed.strip_prefix("# ==== ")?;
            let without_suffix = without_prefix.strip_suffix(" ====")?;
            Some((without_suffix.trim().to_string(), line_no))
        })
        .collect()
}

const ACTIONS_REQUIRING_GENERIC_TAG: &[&str] = &[
    "FilterByTag",
    "TrimAtTag",
    "StoreTagInComment",
    "StoreTagLocationInComment",
    "StoreTagInFastQ",
    "StoreTagInSequence",
    "ReplaceTagWithLetter",
    "QuantifyTag",
    "UppercaseTag",
    "LowercaseTag",
    "StoreTagsInTable",
    "HammingCorrect",
    "ForgetTag",
    "ForgetAllTags",
    "Demultiplex",
    "ConvertToRate",
    "ConvertRegionsToLength",
    "ConcatTags",
];

const ACTIONS_REQUIRING_TWO_TAGS: &[&str] = &["ConcatTags"];

const TAG_DECLARING_CONVERT_STEPS: &[&str] = &["ConvertToRate", "ConvertRegionsToLength"];

#[allow(clippy::too_many_lines)]
fn prep_config_to_parse(extracted_section: &str) -> String {
    // Check if this is a complete configuration that already has [input] and [output] sections
    let has_input_section = extracted_section.contains("[input]");
    let has_output_section = extracted_section.contains("[output]");

    if has_input_section && has_output_section {
        // This is already a complete configuration, return as-is
        return extracted_section.to_string();
    }

    let has_report_step = extracted_section.contains("action = \"Report\"")
        || extracted_section.contains("action = 'Report'");
    let request_report = if has_report_step { "true" } else { "false" };

    let mut config = format!(
        r#"
[input]
read1 = "test_r1.fastq"
read2 = "test_r2.fastq"

[output]
prefix = "output"
format = "fastq"
compression = "raw"
report_json = {request_report}
report_html = false

"#
    )
    .to_string();

    let actions = collect_actions(extracted_section);
    let needs_numeric_tag = actions
        .iter()
        .any(|a| a == "FilterByNumericTag" || a == "EvalExpression");
    let if_tag_present =
        extracted_section.contains("if_tag =") && !extracted_section.contains("#if_tag =");
    let needs_bool_tag = actions.iter().any(|a| {
        a == "FilterByBoolTag" || a == "SwapConditional" || a == "ReverseComplementConditional"
    }) | if_tag_present;
    let needs_generic_tag = actions
        .iter()
        .any(|a| ACTIONS_REQUIRING_GENERIC_TAG.contains(&a.as_str()));
    let needs_two_tags = actions
        .iter()
        .any(|a| ACTIONS_REQUIRING_TWO_TAGS.contains(&a.as_str()));

    let provides_numeric_tag = actions.iter().any(|a| {
        matches!(
            a.as_str(),
            "CalcLength" | "CalcExpectedError" | "ConvertRegionsToLength" | "CalcKmers"
        )
    });
    let provides_bool_tag = actions.iter().any(|a| {
        matches!(
            a.as_str(),
            "TagDuplicates" | "TagOtherFileByName" | "TagOtherFileBySequence"
        )
    });
    /* let provides_any_tag = actions.iter().any(|a| {
        a.starts_with("Extract")
            || a.starts_with("Calc")
            || matches!(
                a.as_str(),
                "TagDuplicates"
                    | "TagOtherFileByName"
                    | "TagOtherFileBySequence"
                    | "HammingCorrect"
            )
    }); */

    // Track which tags we've already created to avoid duplicates
    let mut created_tags = std::collections::HashSet::new();

    if needs_numeric_tag && !provides_numeric_tag {
        config.push_str(
            r#"
                [[step]]
                    action = "CalcLength"
                    segment = "read1"
                    out_label = "mytag"
            "#,
        );
        created_tags.insert("mytag".to_string());
    } else if needs_bool_tag && !provides_bool_tag {
        config.push_str(
            r#"
                [[step]]
                    action = "TagDuplicates"
                    source = "read1"
                    out_label = "mytag"
                    false_positive_rate = 0.0
                    seed = 42
            "#,
        );
        created_tags.insert("mytag".to_string());
    } else if needs_two_tags {
        // Add two tags for transformations that require multiple input tags (e.g., ConcatTags)
        config.push_str(
            r#"
                [[step]]
                    action = "ExtractRegion"
                    segment = "read1"
                    start = 0
                    length = 3
                    out_label = "mytag"
                    anchor = "Start"

                [[step]]
                    action = "ExtractRegion"
                    segment = "read1"
                    start = 3
                    length = 3
                    out_label = "mytag2"
                    anchor = "Start"
            "#,
        );
        created_tags.insert("mytag".to_string());
        created_tags.insert("mytag2".to_string());
    } else if needs_generic_tag {
        // && !provides_any_tag {
        config.push_str(
            r#"
                [[step]]
                    action = "ExtractRegion"
                    segment = "read1"
                    start = 0
                    length = 3
                    out_label = "mytag"
                    anchor = "Start"
            "#,
        );
        created_tags.insert("mytag".to_string());
    }

    // For fragments that use in_label, create a generic tag with that name
    // Skip for actions that don't require pre-existing tags
    let skip_tag_creation = actions.iter().any(|a| {
        matches!(
            a.as_str(),
            "ForgetTag" | "ForgetAllTags" | "StoreTagsInTable"
        )
    });

    // Determine if we need bool or numeric tags based on the action
    let needs_bool_for_in_label = actions.iter().any(|a| {
        matches!(
            a.as_str(),
            "ReverseComplementConditional" | "SwapConditional"
        )
    });

    let needs_numeric_for_in_label = actions
        .iter()
        .any(|a| matches!(a.as_str(), "FilterByNumericTag"));

    if extracted_section.contains("in_label") && !skip_tag_creation {
        // Collect all labels that already exist in the section (from out_label)
        let mut existing_labels = std::collections::HashSet::new();
        for line in extracted_section.lines() {
            if line.contains("out_label") {
                if let Some(start) = line.find("out_label") {
                    let after = &line[start..];
                    if let Some(quote_start) = after.find(['\'', '"']) {
                        let quote_char = after.chars().nth(quote_start).unwrap();
                        let after_quote = &after[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find(quote_char) {
                            existing_labels.insert(after_quote[..quote_end].to_string());
                        }
                    }
                }
            }
        }

        // Extract the label name(s) from in_label fields and create appropriate tags
        for line in extracted_section.lines() {
            if line.contains("in_label") {
                // Try to extract the label value
                if let Some(start) = line.find("in_label") {
                    let after = &line[start..];
                    // Look for quoted string after in_label
                    if let Some(quote_start) = after.find(['\'', '"']) {
                        let quote_char = after.chars().nth(quote_start).unwrap();
                        let after_quote = &after[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find(quote_char) {
                            let label = &after_quote[..quote_end];

                            // Skip if this label is already created in the same block or by us
                            if existing_labels.contains(label) || created_tags.contains(label) {
                                continue;
                            }

                            // Create appropriate tag type based on the action
                            if needs_bool_for_in_label {
                                write!(
                                    &mut config,
                                    r#"
                [[step]]
                    action = "TagDuplicates"
                    source = "read1"
                    out_label = "{label}"
                    false_positive_rate = 0.0
                    seed = 42
            "#
                                )
                                .unwrap();
                                created_tags.insert(label.to_string());
                            } else if needs_numeric_for_in_label {
                                write!(
                                    &mut config,
                                    r#"
                [[step]]
                    action = "CalcLength"
                    segment = "read1"
                    out_label = "{label}"
            "#
                                )
                                .unwrap();
                                created_tags.insert(label.to_string());
                            } else {
                                write!(
                                    &mut config,
                                    r#"
                [[step]]
                    action = "ExtractRegion"
                    segment = "read1"
                    start = 0
                    length = 3
                    out_label = "{label}"
                    anchor = "Start"
            "#
                                )
                                .unwrap();
                                created_tags.insert(label.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Add barcodes section if Demultiplex or HammingCorrect is present
    if actions
        .iter()
        .any(|a| a == "Demultiplex" || a == "HammingCorrect")
        && extracted_section.contains("barcodes = ")
        && !extracted_section.contains("[barcodes.")
    {
        // Extract barcode name from the config
        let barcode_name = if let Some(line) = extracted_section
            .lines()
            .find(|l| l.contains("barcodes = "))
        {
            if let Some(start) = line.find("barcodes = ") {
                let after = &line[start + 11..];
                if let Some(quote_start) = after.find(['\'', '"']) {
                    let quote_char = after.chars().nth(quote_start).unwrap();
                    let after_quote = &after[quote_start + 1..];
                    after_quote
                        .find(quote_char)
                        .map(|quote_end| &after_quote[..quote_end])
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some(name) = barcode_name {
            write!(
                &mut config,
                r"

[barcodes.{name}]
    'AAAAAAAA' = 'sample_1'
    'CCCCCCCC' = 'sample_2'
            ",
            )
            .unwrap();
        }
    }

    config.push_str(extracted_section);

    let declares_tag = actions.iter().any(|a| {
        a.starts_with("Extract")
            || a.starts_with("Calc")
            || TAG_DECLARING_CONVERT_STEPS.contains(&a.as_str())
            || matches!(
                a.as_str(),
                "TagDuplicates"
                    | "TagOtherFileByName"
                    | "TagOtherFileBySequence"
                    | "HammingCorrect"
                    | "EvalExpression"
            )
    });
    let already_stores_tags = actions.iter().any(|a| a == "StoreTagsInTable");
    // Also check if the section contains out_label (for fragments that declare tags)
    // Only count uncommented out_label lines
    let has_out_label = extracted_section.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.starts_with('#') && trimmed.contains("out_label")
    });
    if (declares_tag || has_out_label) && !already_stores_tags {
        config.push_str(
            r#"
                [[step]]
                    action = "StoreTagsInTable"
                    infix = "tags"
                    compression = "Raw"
            "#,
        );
    }
    config
}

/// Get or generate the JSON schema for Transformation enum
/// We extract this from the Config schema since Transformation is not publicly exported
fn get_transformation_schema() -> &'static serde_json::Value {
    TRANSFORMATION_SCHEMA.get_or_init(|| {
        let config_schema = schema_for!(mbf_fastq_processor::config::Config);
        let config_schema_value =
            serde_json::to_value(&config_schema).expect("Failed to convert config schema to JSON");

        // Extract the Transformation schema from the definitions
        let definitions = config_schema_value
            .get("$defs")
            .or_else(|| config_schema_value.get("definitions"))
            .and_then(|d| d.as_object())
            .expect("Config schema does not contain definitions");

        let transformation_schema = definitions
            .get("Transformation")
            .expect("Transformation not found in schema definitions")
            .clone();

        transformation_schema
    })
}

/// Extract field names from the JSON schema for a given transformation variant
/// Returns a map of field name -> list of aliases (empty if no aliases)
fn extract_schema_fields_with_aliases(transformation: &str) -> HashMap<String, Vec<String>> {
    let schema = get_transformation_schema();
    let mut field_map = HashMap::new();

    // Get fields from schema - the oneOf array is at the top level
    let one_ofs = schema
        .get("oneOf")
        .and_then(|o| o.as_array())
        .expect("Schema does not contain oneOf array");

    for variant in one_ofs {
        if let Some(action_const) = variant
            .get("properties")
            .and_then(|p| p.get("action"))
            .and_then(|a| a.get("const"))
            .and_then(|c| c.as_str())
        {
            if action_const == transformation {
                if let Some(properties) = variant.get("properties").and_then(|p| p.as_object()) {
                    for field_name in properties.keys() {
                        if field_name != "action" {
                            field_map.insert(field_name.clone(), Vec::new());
                        }
                    }
                }
                break;
            }
        }
    }

    // Now try to extract aliases from Rust source
    if let Some(aliases_map) = extract_field_aliases_from_source(transformation) {
        for (field, aliases) in aliases_map {
            if let Some(field_aliases) = field_map.get_mut(&field) {
                *field_aliases = aliases;
            }
        }
    }

    field_map
}

/// Extract field aliases from Rust source code for a given transformation
/// Returns a map of `field_name` -> Vec<alias>
fn extract_field_aliases_from_source(transformation: &str) -> Option<HashMap<String, Vec<String>>> {
    // Find the struct file
    let struct_file = find_struct_file_for_transformation(transformation)?;
    let content = fs::read_to_string(&struct_file).ok()?;

    let mut aliases_map: HashMap<String, Vec<String>> = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find the struct definition
    let struct_start = lines
        .iter()
        .position(|line| line.contains("pub struct") && line.contains('{'))?;

    // Look for field definitions and their preceding attributes
    let mut i = struct_start + 1;
    while i < lines.len() {
        let line = lines[i].trim();

        // Stop at closing brace
        if line.starts_with('}') {
            break;
        }

        // Check if this is a field definition
        if line.starts_with("pub ") && line.contains(':') {
            if let Some(field_name) = line.split_whitespace().nth(1) {
                let field_name = field_name.trim_end_matches(':');

                // Look back for alias attributes
                let mut aliases = Vec::new();
                for j in (struct_start + 1..i).rev() {
                    let attr_line = lines[j].trim();

                    // Stop when we hit another pub field or the struct definition
                    if attr_line.starts_with("pub ") || attr_line.contains("pub struct") {
                        break;
                    }

                    // Look for alias attribute
                    if attr_line.contains("#[serde(alias") {
                        if let Some(alias_start) = attr_line.find("alias") {
                            let after_alias = &attr_line[alias_start..];
                            if let Some(quote_start) = after_alias.find('"') {
                                let after_quote = &after_alias[quote_start + 1..];
                                if let Some(quote_end) = after_quote.find('"') {
                                    aliases.push(after_quote[..quote_end].to_string());
                                }
                            }
                        }
                    }
                }

                if !aliases.is_empty() {
                    aliases_map.insert(field_name.to_string(), aliases);
                }
            }
        }

        i += 1;
    }

    Some(aliases_map)
}

/// Find the struct file for a transformation
fn find_struct_file_for_transformation(transformation: &str) -> Option<PathBuf> {
    let transformations_content = fs::read_to_string("src/transformations.rs").ok()?;
    let enum_start = transformations_content.find("pub enum Transformation {")?;
    let content_after_enum = &transformations_content[enum_start..];
    let enum_end = content_after_enum.find("\n}\n")?;
    let enum_content = &content_after_enum[..enum_end];

    for line in enum_content.lines() {
        if line.contains(&format!("{transformation}(")) {
            if let Some(paren_pos) = line.find('(') {
                let after_name = &line[paren_pos + 1..];
                if let Some(paren_close) = after_name.find(')') {
                    let module_path = &after_name[..paren_close];
                    let parts: Vec<&str> = module_path.split("::").collect();

                    if parts.len() == 2 {
                        let struct_name = parts[1];
                        let file_name = struct_name.chars().fold(String::new(), |mut acc, c| {
                            if c.is_uppercase() && !acc.is_empty() {
                                acc.push('_');
                            }
                            acc.push(c.to_ascii_lowercase());
                            acc
                        });

                        let file_path = PathBuf::from(format!(
                            "src/transformations/{}/{}.rs",
                            parts[0], file_name
                        ));

                        if file_path.exists() {
                            return Some(file_path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Check if a field (or any of its aliases) is documented in the given text
fn is_field_in_text(text: &str, field_name: &str, aliases: &[String]) -> bool {
    // Check if the field name or any alias is documented
    let names_to_check = std::iter::once(field_name).chain(aliases.iter().map(String::as_str));

    for name in names_to_check {
        // Look for the name followed by = or :
        // This handles both TOML (field = value) and markdown (field: description)
        let patterns = [
            format!("{name} ="),
            format!("{name}:"),
            format!("`{name}`"),
            format!("**{name}**"),
        ];

        if patterns.iter().any(|pattern| text.contains(pattern)) {
            return true;
        }
    }

    false
}

#[test]
fn test_every_step_has_a_template_section() {
    // Get all transformation names from the enum
    let transformations = get_all_transformations();

    // Read the template file
    let template_content =
        fs::read_to_string("src/template.toml").expect("Failed to read template.toml");

    // Test parsing configuration with each transformation and check target patterns
    let target_patterns = get_transformation_target_patterns();
    let mut errors = Vec::new();
    let transformations_set: HashSet<_> = transformations.iter().cloned().collect();
    let mut documented_sections = HashSet::new();

    for (section_name, line_no) in get_template_section_names(&template_content) {
        if !documented_sections.insert(section_name.clone()) {
            errors.push(format!(
                "Duplicate section {section_name} found in template.toml"
            ));
            continue;
        }

        let extracted_section = match extract_section_from_template(
            &template_content,
            &section_name,
        ) {
            section if section.is_empty() => {
                errors.push(format!(
                        "Failed to extract section for {section_name}, line_no {line_no} from template.toml"
                    ));
                continue;
            }
            section => section,
        };

        // Check target pattern consistency if transformation has a target field
        // Skip deprecated transformations
        if extracted_section.contains("deprecated") {
            // Skip pattern checking for deprecated transformations
        } else if let Some(expected_pattern) = target_patterns.get(&section_name) {
            if !check_target_pattern_in_text(&extracted_section, &section_name, expected_pattern) {
                errors.push(format!(
                    "Template section for {section_name}, line_no {line_no} does not contain the correct target pattern.\nExpected pattern like: {expected_pattern}\nActual section:\n{extracted_section}"
                ));
            }
        }

        let config = prep_config_to_parse(&extracted_section);

        // Verify just the parsing
        match toml::from_str::<mbf_fastq_processor::config::Config>(&config) {
            Ok(mut parsed) => {
                if let Err(e) = parsed.check() {
                    errors.push(format!(
                        "Error in parsing configuration for {section_name}, line_no {line_no}: {e:?}\n{config}"
                    ));
                }
            }
            Err(e) => {
                errors.push(format!(
                    "Could not parse section for {section_name}, line_no {line_no}: {e}.\n{config}"
                ));
            }
        }

        // Check that all struct fields are documented in the template section
        let fields_with_aliases = extract_schema_fields_with_aliases(&section_name);
        for (field, aliases) in &fields_with_aliases {
            if !is_field_in_text(&extracted_section, field, aliases) {
                let alias_info = if aliases.is_empty() {
                    String::new()
                } else {
                    format!(" (or alias: {})", aliases.join(", "))
                };
                errors.push(format!(
                    "Template section for {section_name} is missing field '{field}'{alias_info} (from schema)"
                ));
            }
        }
    }

    let missing: Vec<_> = transformations
        .iter()
        .filter(|transformation| !documented_sections.contains(*transformation))
        .cloned()
        .collect();
    if !missing.is_empty() {
        let mut missing_sorted = missing;
        missing_sorted.sort();
        errors.push(format!(
            "The following transformations are missing in template.toml:\n{}",
            missing_sorted.join(", ")
        ));
    }

    let extra: Vec<_> = documented_sections
        .iter()
        .filter(|section| !transformations_set.contains(*section))
        .cloned()
        .collect();
    if !extra.is_empty() {
        let mut extra_sorted = extra;
        extra_sorted.sort();
        errors.push(format!(
            "The following sections document unknown transformations:\n{}",
            extra_sorted.join(", ")
        ));
    }

    assert!(
        errors.is_empty(),
        "Template validation failed:\n{}",
        errors.join("\n")
    );
}

fn get_all_doc_files() -> Vec<PathBuf> {
    let mut doc_files = Vec::new();

    // Include reference documentation
    let reference_dir = Path::new("../docs/content/docs/reference");
    if reference_dir.exists() {
        visit_dir_recursive(reference_dir, &mut doc_files);
    }

    // Include concept documentation
    let concepts_dir = Path::new("../docs/content/docs/concepts");
    if concepts_dir.exists() {
        visit_dir_recursive(concepts_dir, &mut doc_files);
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
    //verify it's a reference file, not a concept
    if !file_path.components().any(|c| c.as_os_str() == "reference") {
        return None;
    }
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
}

fn extract_toml_from_markdown(
    file_path: &Path,
) -> Result<Option<Vec<(String, usize)>>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    if content.contains("not-a-transformation: true") {
        return Ok(None);
    }

    let mut toml_blocks = Vec::new();
    let mut in_toml_block = false;
    let mut current_block = Vec::new();
    let mut skip_this = false;
    let mut start_line = 0;

    for (line_no, line) in content.lines().enumerate() {
        if line.trim().starts_with("```toml") {
            if line.contains("# ignore_in_test") {
                skip_this = true;
                continue;
            }
            start_line = line_no;
            in_toml_block = true;
            current_block.clear();
        } else if line.trim() == "```" && in_toml_block {
            in_toml_block = false;
            if !current_block.is_empty() && !skip_this {
                toml_blocks.push((current_block.join("\n"), start_line));
            }
            skip_this = false;
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
#[allow(clippy::too_many_lines)]
fn test_documentation_toml_examples_parse() {
    let doc_files = get_all_doc_files();
    let mut failed_files = Vec::new();

    for doc_file in &doc_files {
        let transformation = extract_transformation_from_filename(doc_file);
        let ignored = ["CalcMeanQuality.md"];

        // Read the markdown content once for field checking
        let markdown_content = match fs::read_to_string(doc_file) {
            Ok(content) => content,
            Err(e) => {
                failed_files.push(format!(
                    "{}: Failed to read file: {}",
                    doc_file.display(),
                    e
                ));
                continue;
            }
        };

        // Check that all struct fields are documented in the markdown
        if let Some(transformation) = &transformation {
            if !markdown_content.contains("not-a-transformation: true") {
                let fields_with_aliases = extract_schema_fields_with_aliases(&transformation);
                for (field, aliases) in &fields_with_aliases {
                    if !is_field_in_text(&markdown_content, field, aliases) {
                        let alias_info = if aliases.is_empty() {
                            String::new()
                        } else {
                            format!(" (or alias: {})", aliases.join(", "))
                        };
                        failed_files.push(format!(
                        "{}: Documentation is missing field '{field}'{alias_info} (from schema)",
                        doc_file.display()
                    ));
                    }
                }
            }
        }

        match extract_toml_from_markdown(doc_file) {
            Ok(Some(toml_blocks)) => {
                if toml_blocks.is_empty()
                    && !ignored.contains(&doc_file.file_name().and_then(|o| o.to_str()).unwrap())
                    && !doc_file.components().any(|c| c.as_os_str() == "concepts")
                {
                    failed_files.push(format!("{}: No TOML examples found", doc_file.display()));
                    continue;
                }

                let target_patterns = get_transformation_target_patterns();

                for (i, (toml_block, start_line_no)) in toml_blocks.iter().enumerate() {
                    if toml_block.contains("# ignore_in_test") {
                        continue;
                    }

                    // Skip validation for concept files that contain TOML fragments rather than complete configurations
                    let is_concept_file =
                        doc_file.components().any(|c| c.as_os_str() == "concepts");

                    // For concept files, skip the specific transformation matching since they contain examples
                    // using multiple transformations, but still validate TOML parsing
                    if let Some(transformation) = &transformation {
                        if !is_concept_file
                            && !toml_block.contains(&format!("action = \"{transformation}\""))
                        {
                            failed_files.push(format!(
                                "{}: TOML block {} does not contain action = \"{transformation}\"",
                                doc_file.display(),
                                i + 1,
                            ));
                            continue;
                        }

                        // Check target pattern consistency if transformation has a target field
                        // Skip this check for concept files since they contain examples using multiple transformations
                        if !is_concept_file {
                            if let Some(expected_pattern) = target_patterns.get(&transformation[..])
                            {
                                if !check_target_pattern_in_text(
                                    toml_block,
                                    &transformation,
                                    expected_pattern,
                                ) {
                                    failed_files.push(format!(
                                    "{}: TOML block {}, line: {start_line_no} does not contain the correct target pattern.\nExpected pattern like: {}\nActual block:\n{}",
                                    doc_file.display(),
                                    i + 1,
                                    expected_pattern,
                                    toml_block
                                ));
                                }
                            }
                        }
                    }

                    let config = prep_config_to_parse(toml_block);

                    // Try to parse the configuration
                    match toml::from_str::<mbf_fastq_processor::config::Config>(&config) {
                        Ok(mut parsed_config) => {
                            if let Err(e) = parsed_config.check() {
                                failed_files.push(format!(
                                    "{}: TOML block {}, line: {start_line_no} failed validation: {:?}\n{}",
                                    doc_file.display(),
                                    i + 1,
                                    e,
                                    config,
                                ));
                            }
                        }
                        Err(e) => {
                            failed_files.push(format!(
                                "{}: TOML block {}, line: {start_line_no} failed to parse: {}",
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

#[test]
fn test_llm_guide_covers_all_transformations() {
    let transformations = get_all_transformations();
    let llm_guide_path = Path::new("../docs/content/docs/reference/llm-guide.md");

    // Check if the file exists
    assert!(
        llm_guide_path.exists(),
        "LLM guide not found at {}",
        llm_guide_path.display()
    );

    // Read the LLM guide
    let llm_guide_content =
        fs::read_to_string(llm_guide_path).expect("Failed to read llm-guide.md");

    let mut errors = Vec::new();
    let mut documented_transformations = HashSet::new();

    // Check for each transformation in the LLM guide
    for transformation in &transformations {
        // Look for the transformation name in various contexts:
        // 1. As a heading: "### TransformationName"
        // 2. In action field: action = 'TransformationName'
        // 3. As a step reference
        let heading_pattern = format!("### {transformation}");
        let action_pattern_single = format!("action = '{transformation}'");
        let action_pattern_double = format!("action = \"{transformation}\"");

        if llm_guide_content.contains(&heading_pattern)
            || llm_guide_content.contains(&action_pattern_single)
            || llm_guide_content.contains(&action_pattern_double)
        {
            documented_transformations.insert(transformation.clone());
        } else {
            errors.push(format!(
                "Transformation '{transformation}' is not documented in llm-guide.md"
            ));
        }
    }

    // Report missing transformations
    if !errors.is_empty() {
        let mut missing_sorted = errors;
        missing_sorted.sort();
        panic!(
            "LLM guide validation failed:\n{}",
            missing_sorted.join("\n")
        );
    }

    // Verify we found a reasonable number of transformations
    assert!(
        documented_transformations.len() == transformations.len(),
        "LLM guide coverage is too low: (documented {}/{} transformations)",
        documented_transformations.len(),
        transformations.len()
    );
}

fn extract_toml_blocks_from_llm_guide(content: &str) -> Vec<(String, usize)> {
    let mut toml_blocks = Vec::new();
    let mut in_toml_block = false;
    let mut current_block = Vec::new();
    let mut start_line_no = 0;

    for (lineno, line) in content.lines().enumerate() {
        if line.trim() == "```toml" {
            in_toml_block = true;
            current_block.clear();
            start_line_no = lineno;
        } else if line.trim() == "```" && in_toml_block {
            in_toml_block = false;
            if !current_block.is_empty() {
                toml_blocks.push((current_block.join("\n"), start_line_no));
            }
        } else if in_toml_block {
            current_block.push(line);
        }
    }

    toml_blocks
}

#[test]
fn test_llm_guide_toml_examples_parse() {
    let llm_guide_path = Path::new("../docs/content/docs/reference/llm-guide.md");

    assert!(
        llm_guide_path.exists(),
        "LLM guide not found at {}",
        llm_guide_path.display()
    );

    let llm_guide_content =
        fs::read_to_string(llm_guide_path).expect("Failed to read llm-guide.md");

    let toml_blocks = extract_toml_blocks_from_llm_guide(&llm_guide_content);
    let mut failed_examples = Vec::new();

    for (i, (toml_block, line_no)) in toml_blocks.iter().enumerate() {
        // Skip blocks that are marked as fragments or incomplete
        if toml_block.contains("# fragment")
            || toml_block.contains("# incomplete")
            || toml_block.contains("# example-only")
        {
            continue;
        }

        // Check if this is a complete configuration (has [input] and [output])
        let has_input = toml_block.contains("[input]");
        let has_output = toml_block.contains("[output]");

        if !has_input || !has_output {
            // This is a partial example, wrap it with minimal config
            let config = prep_config_to_parse(toml_block);

            match toml::from_str::<mbf_fastq_processor::config::Config>(&config) {
                Ok(mut parsed_config) => {
                    if let Err(e) = parsed_config.check() {
                        failed_examples.push(format!(
                            "LLM guide TOML block {} , line_no {line_no}failed validation: {:?}\nBlock:\n{}",
                            i + 1,
                            e,
                            toml_block
                        ));
                    }
                }
                Err(e) => {
                    failed_examples.push(format!(
                        "LLM guide TOML block {}, line_no {line_no} failed to parse: {}\nBlock:\n{}",
                        i + 1,
                        e,
                        toml_block
                    ));
                }
            }
        } else {
            // This is a complete configuration, parse directly
            match toml::from_str::<mbf_fastq_processor::config::Config>(toml_block) {
                Ok(mut parsed_config) => {
                    if let Err(e) = parsed_config.check() {
                        failed_examples.push(format!(
                            "LLM guide complete config block {}, line_no {line_no} failed validation: {:?}\nBlock:\n{}",
                            i + 1,
                            e,
                            toml_block
                        ));
                    }
                }
                Err(e) => {
                    failed_examples.push(format!(
                        "LLM guide complete config block {}, line_no {line_no} failed to parse: {}\nBlock:\n{}",
                        i + 1,
                        e,
                        toml_block
                    ));
                }
            }
        }
    }

    assert!(
        failed_examples.is_empty(),
        "LLM guide TOML examples validation failed:\n{}",
        failed_examples.join("\n\n")
    );
}

#[test]
fn test_hugo_builds_documentation_site() {
    let temp_destination =
        tempdir().expect("Failed to allocate temporary directory for Hugo output");
    let mut command = Command::new("hugo");
    command
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .arg("--source")
        .arg("../docs")
        .arg("--destination")
        .arg(temp_destination.path())
        .arg("--panicOnWarning")
        //.arg("--quiet")
        .env("HUGO_ENVIRONMENT", "production")
        .env("HUGO_ENV", "production");

    let output = match command.output() {
        Ok(output) => output,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            eprintln!("Skipping Hugo documentation build test: `hugo` binary not found in PATH.");
            return;
        }
        Err(error) => panic!("Failed to execute `hugo`: {error}"),
    };

    assert!(
        output.status.success(),
        "Hugo failed to build documentation (status {}).\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn test_flake_rust_version_matches_msrv() {
    // Verify that the Rust version used in flake.nix is >= the MSRV declared in Cargo.toml
    // This prevents accidentally building with an older Rust than our declared minimum.

    // Read Cargo.toml and extract rust-version
    let cargo_toml_path = Path::new("../Cargo.toml");
    let cargo_content = fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");

    let msrv = cargo_content
        .lines()
        .find(|line| line.trim().starts_with("rust-version"))
        .and_then(|line| {
            let after_eq = line.split('=').nth(1)?;
            let trimmed = after_eq.trim().trim_matches('"');
            Some(trimmed.to_string())
        })
        .expect("Could not find rust-version in Cargo.toml");

    // Read flake.nix and extract Rust version
    let flake_path = Path::new("../flake.nix");
    let flake_content = fs::read_to_string(flake_path).expect("Failed to read flake.nix");

    // Look for pattern like: rust = pkgs.rust-bin.stable."1.90.0".default
    let flake_rust_version = flake_content
        .lines()
        .find(|line| {
            line.contains("rust-bin.stable.") && line.contains("default")
        })
        .and_then(|line| {
            // Extract version between quotes after "stable."
            let after_stable = line.split("stable.").nth(1)?;
            let version_start = after_stable.find('"')? + 1;
            let after_first_quote = &after_stable[version_start..];
            let version_end = after_first_quote.find('"')?;
            Some(after_first_quote[..version_end].to_string())
        })
        .expect("Could not find rust-bin.stable version in flake.nix");

    // Parse versions for comparison (major.minor.patch)
    fn parse_version(v: &str) -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    }

    let msrv_parsed = parse_version(&msrv);
    let flake_parsed = parse_version(&flake_rust_version);

    assert!(
        flake_parsed >= msrv_parsed,
        "flake.nix uses Rust {flake_rust_version} but Cargo.toml declares rust-version = \"{msrv}\". \
         The Nix flake must use a Rust version >= the declared MSRV."
    );

    println!("✓ flake.nix Rust version ({flake_rust_version}) >= MSRV ({msrv})");
}

#[test]
fn test_readme_toml_examples_validate() {
    // This test extracts TOML code blocks from README.md and validates them
    use mbf_fastq_processor::config::Config;
    use std::fs;
    use std::path::Path;

    let readme_path = Path::new("../README.md");
    assert!(readme_path.exists(), "README.md not found");

    let readme_content = fs::read_to_string(readme_path).expect("Failed to read README.md");

    // Extract TOML code blocks (between ```toml and ```)
    let mut toml_blocks = Vec::new();
    let mut in_toml_block = false;
    let mut current_block = String::new();
    let mut block_start_line = 0;
    let mut line_num = 0;

    for line in readme_content.lines() {
        line_num += 1;
        if line.trim().starts_with("```toml") {
            in_toml_block = true;
            current_block.clear();
            block_start_line = line_num;
        } else if line.trim().starts_with("```") && in_toml_block {
            in_toml_block = false;
            if !current_block.trim().is_empty() {
                toml_blocks.push((block_start_line, current_block.clone()));
            }
        } else if in_toml_block {
            current_block.push_str(line);
            current_block.push('\n');
        }
    }

    assert!(
        !toml_blocks.is_empty(),
        "No TOML code blocks found in README.md"
    );

    println!("\n✓ Found {} TOML block(s) in README.md", toml_blocks.len());

    // Validate each TOML block using the same approach as the run() function
    for (line_no, toml_content) in &toml_blocks {
        println!("  Validating TOML block starting at line {line_no}...");

        // Parse the TOML using eserde (same as in run())
        let mut parsed = match eserde::toml::from_str::<Config>(toml_content) {
            Ok(config) => config,
            Err(e) => {
                panic!("README.md TOML block at line {line_no} failed to parse:\n{e:?}",);
            }
        };

        // Validate the config using check() (same as in run())
        // Note: This will fail on input file validation since files don't exist,
        // but it will catch TOML syntax errors and structural issues
        match parsed.check() {
            Ok(()) => {
                println!("    ✓ TOML block at line {line_no} validated successfully",);
            }
            Err(e) => {
                let error_msg = format!("{e:?}");
                // Allow errors about missing input files, but catch everything else
                if error_msg.contains("Could not read")
                    || error_msg.contains("No such file")
                    || error_msg.contains("does not exist")
                {
                    println!(
                        "    ✓ TOML block at line {line_no} validated (structure valid, expected file errors ignored)",
                    );
                } else {
                    panic!(
                        "README.md TOML block at line {line_no} failed validation:\n{error_msg}",
                    );
                }
            }
        }
    }

    println!("\n✓ All README.md TOML examples are valid!");
}
