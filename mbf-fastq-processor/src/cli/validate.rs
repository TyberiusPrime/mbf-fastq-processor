use anyhow::{Context, Result};
use ex::fs;
use std::path::Path;

use crate::config::Config;
use toml_pretty_deser::prelude::*;

pub fn validate_config(toml_file: &Path) -> Result<Vec<String>> {
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let result = Config::tpd_from_toml(&raw_config, FieldMatchMode::AnyCase, VecMode::SingleOk);
    let checked = match result {
        Ok(config) => config,
        Err(e) => {
            return Err(anyhow::anyhow!("{}", e.pretty("config.toml")));
        }
    };
    let checked = checked.check_for_validation()?;

    let toml_dir = toml_file.parent().unwrap_or_else(|| Path::new("."));

    let mut warnings = Vec::new();

    match &checked.input.structured {
        crate::config::StructuredInput::Interleaved { files, .. } => {
            for file in files {
                if file != crate::config::STDIN_MAGIC_PATH {
                    let file_path = toml_dir.join(file);
                    if fs::metadata(&file_path).is_err() {
                        warnings.push(format!("Input file not found: {file}"));
                    }
                }
            }
        }
        crate::config::StructuredInput::Segmented { segment_files, .. } => {
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
    }

    Ok(warnings)
}
