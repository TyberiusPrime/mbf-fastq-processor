use regex::Regex;

pub mod process;
pub mod validate;
pub mod verify;

pub(crate) fn improve_error_messages(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
    let mut e = extend_with_step_annotation(e, raw_toml);
    let msg = format!("{e:?}");
    let barcode_regexp = Regex::new("barcodes.[^:]+: invalid type: sequence,")
        .expect("hardcoded regex pattern is valid");
    if barcode_regexp.is_match(&msg) {
        e = e.context("Use `[barcode.<name>]` instead of `[[barcode.<name>]]` in your config");
    }
    let options_search = "options[0]: invalid type: map, expected usize";
    if msg.contains(options_search) {
        e = e.context(
            "The 'options' field should be a table, not an array. Use [options], not [[options]]",
        );
    }
    let mistyped_input = "invalid type: sequence, expected struct __ImplEDeserializeForInput";
    if msg.contains(mistyped_input) {
        e = e.context("(input): The 'input' section should be a table, not an array. Use [input] instead of [[input]]");
    } else {
        let mistyped_input = "expected struct __ImplEDeserializeForInput";
        if msg.contains(mistyped_input) {
            e = e.context("(input): The 'input' section should be a table of segment = [filenames,...]. Example:\n[input]\nread1 = 'filename.fq'");
        }
    }
    let nested_input = "input: invalid type: map, expected string or list of strings";
    if msg.contains(nested_input) {
        e = e.context("x.y as key in TOML means 'a map below the current [section]. You are probably trying for a segment name with a dot (not allowed, remove dot), or tried [input] output.prefix, but you need [output]");
    }
    e
}

fn extend_with_step_annotation(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
    let msg = format!("{e:?}");
    let step_regex = Regex::new(r"step.(\d+).").expect("hardcoded regex pattern is valid");
    if let Some(matches) = step_regex.captures(&msg) {
        let step_no = &matches[1];
        let step_int = step_no.parse::<usize>().unwrap_or(0);
        let parsed = toml::from_str::<toml::Value>(raw_toml);
        if let Ok(parsed) = parsed
            && let Some(step) = parsed.get("step")
            && let Some(steps) = step.as_array()
            && let Some(step_no_i) = steps.get(step_int)
            && let Some(action) = step_no_i.get("action").and_then(|v| v.as_str())
        {
            return e.context(format!(
                "Error in Step {step_no} (0-based), action = {action}"
            ));
        }
    }
    e
}
