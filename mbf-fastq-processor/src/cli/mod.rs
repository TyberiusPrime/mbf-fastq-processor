use mbf_fastq_processor_steps::link_docs;
use toml_pretty_deser::TomlValue;

use crate::config::PartialConfig;

pub mod process;
pub mod validate;
pub mod verify;

pub(crate) fn improve_error_messages(
    toml_filename: &str,
    mut err: toml_pretty_deser::DeserError<PartialConfig>,
) -> String {
    fn add_help<T>(toml_value: &mut TomlValue<T>, step_name: &str) {
        let new_help = format!(
            "See {}\nOr run: `{} template {}` for more information.",
            link_docs(step_name),
            env!("CARGO_PKG_NAME"),
            step_name
        );
        toml_value.help = match toml_value.help.as_ref() {
            Some(old_help) => Some(format!("{old_help}\n{new_help}",)),
            None => Some(new_help),
        };
        if let Some(context) = toml_value.context.as_mut() {
            context.1 = "In this step".to_string();
        }
    }
    match &mut err {
        toml_pretty_deser::DeserError::ParsingFailure(_, _) => {}
        toml_pretty_deser::DeserError::DeserFailure(_source, tv_partial) => {
            if let Some(partial) = tv_partial.value.as_mut()
                && let Some(steps) = partial.transform.value.as_mut()
            {
                for tv_step in steps.iter_mut() {
                    if !tv_step.is_ok()
                        && let Some(step) = tv_step.value.as_ref()
                    {
                        let step_name = step.tpd_get_tag();
                        add_help(tv_step, step_name);
                    }
                }
                if !partial.input.is_ok() {
                    add_help(&mut partial.input, "input-section");
                }
            } // cov:excl-line
        }
    }
    err.pretty(toml_filename)
}
