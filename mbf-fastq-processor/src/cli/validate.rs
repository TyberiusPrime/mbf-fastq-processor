use anyhow::{Context, Result};
use ex::fs;
use toml_edit::Document;
use std::path::Path;

use crate::cli;
use crate::config::Config;
use crate::config::deser::{ErrorCollectorExt, FromToml};

pub fn validate_config(toml_file: &Path) -> Result<Vec<String>> {
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;

    let tomled = raw_config
        .parse::<Document<String>>()
        .context("Failed to parse TOML syntax.")?;
    let mut collector = crate::config::deser::new_error_collector();
    let parsed = Config::from_toml(tomled.as_item(), &mut collector).map_err(|_| {
        anyhow::anyhow!(collector.render(&raw_config, &toml_file.display().to_string()))
    });
    let checked = parsed
        .map_err(|e| cli::improve_error_messages(e.into(), &raw_config))
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?
        .check_for_validation()?;

    let toml_dir = toml_file.parent().unwrap_or_else(|| Path::new("."));

    let mut warnings = Vec::new();

    match &checked.input.structured {
        Some(crate::config::StructuredInput::Interleaved { files, .. }) => {
            for file in files {
                if file != crate::config::STDIN_MAGIC_PATH {
                    let file_path = toml_dir.join(file);
                    if fs::metadata(&file_path).is_err() {
                        warnings.push(format!("Input file not found: {file}"));
                    }
                }
            }
        }
        Some(crate::config::StructuredInput::Segmented { segment_files, .. }) => {
            for (segment_name, files) in segment_files {
                for file in files {
                    if file != crate::config::STDIN_MAGIC_PATH {
                        let file_path = toml_dir.join(file);
                        if fs::metadata(&file_path).is_err() {
                            warnings.push(format!(
                                "Input file not found in segment '{segment_name}': {file}"
                            ));
                        }
                    }
                }
            }
        }
        None => {}
    }

    Ok(warnings)
}
