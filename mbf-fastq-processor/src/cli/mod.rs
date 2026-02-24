//use regex::Regex;

use crate::config::PartialConfig;

pub mod process;
pub mod validate;
pub mod verify;

pub(crate) fn improve_error_messages(
    toml_filename: &str,
    mut err: toml_pretty_deser::DeserError<PartialConfig>,
) -> String {
    let doc_url = format!(
        "{}v{}/docs/reference/",
        env!("CARGO_PKG_HOMEPAGE"),
        env!("CARGO_PKG_VERSION")
    );

    match &mut err {
        toml_pretty_deser::DeserError::ParsingFailure(_, _) => {}
        toml_pretty_deser::DeserError::DeserFailure(_source, tv_partial) => {
            if let Some(partial) = tv_partial.value.as_mut() {
                if let Some(Some(steps)) = partial.transform.value.as_mut() {
                    for tv_step in steps.iter_mut() {
                        if tv_step.is_nested()
                            && let Some(step) = tv_step.value.as_ref()
                        {
                            let step_name = step.tpd_get_tag();
                            tv_step.help = Some(format!("See {doc_url}{step_name}"));
                        }
                    }
                }
            }
        }
    }
    let pretty = err.pretty(toml_filename);
    pretty
    //     let regex = Regex::new(r#"(?m)action\s=\s['"]([a-zA-Z09-]+)['"]\s*$"#)
    //         .expect("hardcoded regex pattern is invalid");
    //     regex
    //         .replace_all(
    //             pretty_str,
    //             "$0
    // #   See https://tyberiusprime.github.io/mbf-fastq-processor/main/docs/reference/$1",
    //         )
    //         .to_string()
}

// fn extend_with_step_annotation(e: anyhow::Error, raw_toml: &str) -> anyhow::Error {
//     let msg = format!("{e:?}");
//     let step_regex = Regex::new(r"step.(\d+).").expect("hardcoded regex pattern is valid");
//     if let Some(matches) = step_regex.captures(&msg) {
//         let step_no = &matches[1];
//         let step_int = step_no.parse::<usize>().unwrap_or(0);
//         let parsed = toml::from_str::<toml::Value>(raw_toml);
//         if let Ok(parsed) = parsed
//             && let Some(step) = parsed.get("step")
//             && let Some(steps) = step.as_array()
//             && let Some(step_no_i) = steps.get(step_int)
//             && let Some(action) = step_no_i.get("action").and_then(|v| v.as_str())
//         {
//             return e.context(format!(
//                 "Error in Step {step_no} (0-based), action = {action}"
//             ));
//         }
//     }
//     e
// }
