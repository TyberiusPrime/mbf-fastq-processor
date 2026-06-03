use anyhow::Result;
use std::path::Path;
use toml_pretty_deser::prelude::*;

use fastqrab_io::STDIN_MAGIC_PATH;

use crate::{cli::improve_error_messages, config::Config};

pub fn list_config_output_files(toml_file: &Path) -> Result<Vec<String>> {
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
    let checked = checked.check_for_validation()?;
    if toml_file == Path::new("-") && crate::cli::config_uses_stdin_fastq(&checked.input.structured)
    {
        anyhow::bail!(
            "Cannot read configuration from stdin ('-') when the configuration also uses stdin \
             ('{STDIN_MAGIC_PATH}') for FASTQ input. Use a config file on disk instead."
        );
    }

    //let current_dir_buf;
    // let toml_dir = if toml_file == Path::new("-") {
    //     current_dir_buf = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    //     current_dir_buf.as_path()
    // } else {
    //     toml_file.parent().unwrap_or_else(|| Path::new("."))
    // };

    let mut output_files = Vec::new();
    let ix_sep = checked
        .output
        .as_ref()
        .map(|x| x.ix_separator.as_str())
        .unwrap_or("_");
    let prefix = checked
        .output
        .as_ref()
        .map(|x| x.prefix.as_str())
        .expect("How did it pass validation without prefix");
    for declaration in checked.output_declarations_per_transformation {
        if let Some(declaration) = declaration {
            for a_file in declaration {
                output_files.push(match a_file.target {
                    fastqrab_io::io::output::chunked_writer::WriteTargetConfig::File(
                        path_config,
                    ) => {
                        let mut parts = vec![prefix];
                        parts.extend(path_config.infix_parts().iter().map(|s| s.as_str()));
                        format!(
                            "{}.{}",
                            fastqrab_steps::join_nonempty(parts, ix_sep),
                            path_config.suffix()
                        )
                    }
                    fastqrab_io::io::output::chunked_writer::WriteTargetConfig::Stdout => {
                        "--stdout--".to_string()
                    }
                });
            }
        }
    }

    Ok(output_files)
}
