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
            if let Some(partial) = tv_partial.value.as_mut()
                && let Some(steps) = partial.transform.value.as_mut()
            {
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
    err.pretty(toml_filename)
}
