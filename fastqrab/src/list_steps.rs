use schemars::schema_for;
use std::fmt::Write;

use crate::transformations::Transformation;

/// List all available transformation steps with their descriptions
///
/// # Panics
/// If Schema fails?
#[must_use]
pub fn list_steps() -> Vec<(String, String)> {
    let schema = schema_for!(Transformation);
    let mut steps: Vec<(String, String)> = Vec::new();

    let one_ofs = schema
        .as_object()
        .expect("schema_for! always produces an object")
        .get("oneOf")
        .expect("Transformation schema must have oneOf field");
    let defs = schema.get("$defs").expect("No defs");
    for entry in one_ofs
        .as_array()
        .expect("oneOf field in schema must be an array")
    {
        // `Transformation` is internally tagged on `action`, so each oneOf
        // branch carries the variant name in `properties.action.const` and a
        // sibling `$ref` pointing at the step's own `$def`.
        let action_kind = entry
            .get("properties")
            .expect("No props")
            .get("action")
            .expect("No action discriminator")
            .get("const")
            .expect("action is not a const")
            .as_str()
            .expect("action const must be a string");
        let inner_kind = entry
            .get("$ref")
            .expect("no $ref")
            .as_str()
            .expect("inner kind not a string")
            .rsplit('/')
            .next()
            .expect("no / in $ref");
        let description = defs
            .get(inner_kind)
            .and_then(|x| x.get("description"))
            .and_then(|x| x.as_str())
            .unwrap_or("");
        steps.push((action_kind.to_string(), description.to_string()));
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
        // Get first line of description only
        let first_line = description.lines().next().unwrap_or("");
        writeln!(&mut output, "  {action:<30} {first_line}")
            .expect("writing to String never fails");
    }

    output
}
