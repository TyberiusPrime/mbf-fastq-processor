use crate::transformations::Transformation;
use schemars::schema_for;
use schemars::schema::Schema;

/// List all available transformation steps with their descriptions
pub fn list_steps() -> Vec<(String, String)> {
    let schema = schema_for!(Transformation);
    let mut steps = Vec::new();

    // Extract the enum variants from the schema
    let schema_object = &schema.schema;
    if let Some(subschemas) = &schema_object.subschemas {
        if let Some(one_of) = &subschemas.one_of {
            for subschema in one_of {
                if let Schema::Object(obj) = subschema {
                    // Get the action name from properties
                    if let Some(Schema::Object(action_schema)) = obj.object.as_ref().and_then(|o| o.properties.get("action")) {
                        if let Some(enum_values) = &action_schema.enum_values {
                            if let Some(action_value) = enum_values.first() {
                                let action_name = action_value
                                    .as_str()
                                    .unwrap_or("Unknown")
                                    .to_string();

                                // Get description from metadata
                                let description = obj
                                    .metadata
                                    .as_ref()
                                    .and_then(|m| m.description.clone())
                                    .unwrap_or_default();

                                steps.push((action_name, description));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by action name
    steps.sort_by(|a, b| a.0.cmp(&b.0));
    steps
}

/// Format steps for display
pub fn format_steps_list() -> String {
    let steps = list_steps();
    let mut output = String::from("Available transformation steps:\n\n");

    for (action, description) in steps {
        if description.is_empty() {
            output.push_str(&format!("  {action}\n"));
        } else {
            // Get first line of description only
            let first_line = description.lines().next().unwrap_or("");
            output.push_str(&format!("  {action:<30} {first_line}\n"));
        }
    }

    output
}
