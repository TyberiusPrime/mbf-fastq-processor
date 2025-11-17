use crate::transformations::Transformation;
use schemars::schema_for;
use std::fmt::Write;

/// List all available transformation steps with their descriptions
#[must_use]
pub fn list_steps() -> Vec<(String, String)> {
    let schema = schema_for!(Transformation);
    let mut steps: Vec<(String, String)> = Vec::new();

    let one_ofs = schema.as_object().unwrap().get("oneOf").unwrap();
    for entry in one_ofs.as_array().unwrap() {
        let action = entry
            .as_object()
            .unwrap()
            .get("properties")
            .unwrap()
            .get("action");
        if let Some(action) = action {
            if let Some(str) = action.get("const").and_then(|x| x.as_str()) {
                let desc = entry
                    .as_object()
                    .unwrap()
                    .get("description")
                    .and_then(|x| x.as_str())
                    .unwrap_or("");
                steps.push((str.to_string(), desc.to_string()));
            }
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
            write!(&mut output, "  {action}\n");
        } else {
            // Get first line of description only
            let first_line = description.lines().next().unwrap_or("");
            write!(&mut output, "  {action:<30} {first_line}\n");
        }
    }

    output
}
