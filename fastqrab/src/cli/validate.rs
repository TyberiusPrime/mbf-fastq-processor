use anyhow::Result;
use ex::fs;
use std::path::Path;
use toml_pretty_deser::prelude::*;

use fastqrab_io::STDIN_MAGIC_PATH;

use crate::{cli::improve_error_messages, config::Config};

pub fn validate_config(toml_file: &Path, was_stdin: bool) -> Result<Vec<String>> {
    let raw_config = crate::cli::read_config_raw(toml_file)?;
    let result = Config::tpd_from_toml(&raw_config, FieldMatchMode::AnyCase, VecMode::SingleOk);
    let checked = match result {
        Ok(config) => config,
        Err(e) => {
            //dbg!(&e);
            return Err(anyhow::anyhow!(
                "{}",
                improve_error_messages("config.toml", e)
            ));
        }
    };
    let checked = checked.check_for_validation(was_stdin)?;

    let current_dir_buf;
    let toml_dir = if toml_file == Path::new("-") {
        current_dir_buf = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        current_dir_buf.as_path()
    } else {
        toml_file.parent().unwrap_or_else(|| Path::new("."))
    };

    let mut warnings = Vec::new();

    match &checked.input.structured {
        crate::config::StructuredInput::Interleaved { files, .. } => {
            for file in files {
                if file != STDIN_MAGIC_PATH {
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
                    if file != STDIN_MAGIC_PATH {
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
