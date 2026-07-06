use anyhow::{Context, Result};
use fastqrab_steps::link_docs;
use std::path::Path;
use toml_pretty_deser::TomlValue;

use crate::config::PartialConfig;

pub mod main;
pub mod output_files;
pub mod process;
pub mod validate;
pub mod verify;

/// Read the raw config from a file path or from stdin if path is `-`.
pub(crate) fn read_config_raw(path: &Path) -> Result<String> {
    if path == Path::new("-") {
        use std::io::Read;
        let mut content = String::new();
        std::io::stdin()
            .read_to_string(&mut content)
            .context("Failed to read configuration from stdin")?;
        Ok(content)
    } else {
        ex::fs::read_to_string(path)
            .with_context(|| format!("Could not read toml file: {}", path.to_string_lossy()))
    }
}


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
