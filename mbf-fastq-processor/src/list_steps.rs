use crate::transformations::Transformation;
use schemars::schema_for;
use std::fmt::Write;

/// List all available transformation steps with their descriptions
#[must_use]
pub fn list_steps() -> Vec<(String, String)> {
    let schema = schema_for!(Transformation);
    let mut steps: Vec<(String, String)> = Vec::new();

    let one_ofs = schema
        .as_object()
        .expect("schema_for! always produces an object")
        .get("oneOf")
        .expect("Transformation schema must have oneOf field");
    for entry in one_ofs
        .as_array()
        .expect("oneOf field in schema must be an array")
    {
        let action = entry
            .as_object()
            .expect("each oneOf entry must be an object")
            .get("properties")
            .expect("each transformation variant must have properties")
            .get("action");
        if let Some(action) = action
            && let Some(str) = action.get("const").and_then(|x| x.as_str())
        {
            let desc = entry
                .as_object()
                .expect("each oneOf entry must be an object")
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("");
            steps.push((str.to_string(), desc.to_string()));
        }
    }

    // Sort by action name
    steps.sort_by(|a, b| a.0.cmp(&b.0));
    steps
}

/// Format steps for display
#[must_use]
pub fn format_steps_list() -> String {
    let steps = list_steps();
    let mut output = String::from("Available transformation steps:\n\n");

    for (action, description) in steps {
        if description.is_empty() {
            writeln!(&mut output, "  {action}").expect("writing to String never fails");
        } else {
            // Get first line of description only
            let first_line = description.lines().next().unwrap_or("");
            writeln!(&mut output, "  {action:<30} {first_line}")
                .expect("writing to String never fails");
        }
    }

    output
}
