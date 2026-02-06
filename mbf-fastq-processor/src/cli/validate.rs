use anyhow::{Context, Result};
use ex::fs;
use std::path::Path;

use crate::cli;
use crate::config::Config;
use toml_pretty_deser::{deserialize_with_mode, DeserError, FieldMatchMode, VecMode};

pub fn validate_config(toml_file: &Path) -> Result<Vec<String>> {
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let result = deserialize_with_mode::<PartialConfig, Config>(
        &raw_config,
        FieldMatchMode::CaseInsensitive,
        VecMode::SingleToVec,
    );
    let checked = match result {
        Ok(config) => config,
        Err(e) => {
            return Err(anyhow::anyhow!("{}", e.pretty(&raw_config, "config.toml")));
        }
    };
    checked.check_for_validation()?;

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
