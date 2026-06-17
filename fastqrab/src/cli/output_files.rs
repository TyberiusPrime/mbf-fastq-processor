use anyhow::Result;
use std::path::Path;
use toml_pretty_deser::prelude::*;

use fastqrab_io::STDIN_MAGIC_PATH;

use crate::pipeline::{DemultiplexChain, ResolvedOutputName, enumerate_declaration_outputs};
use crate::transformations::Transformation;
use crate::{cli::improve_error_messages, config::Config};

/// The result of [`list_config_output_files`].
pub struct OutputFilesListing {
    /// The files the configuration would produce, in pipeline order.
    pub files: Vec<String>,
    /// True if at least one listed file belongs to a chunked output. For those,
    /// only the first chunk (`.0`) is listed; the run may emit further numbered
    /// chunks (`.1`, `.2`, …) depending on data volume.
    pub any_chunked: bool,
}

/// List the output files a (valid) config would produce, mirroring exactly what
/// the runtime would write. This walks the stages the same way the pipeline does
/// — accumulating the demultiplex tag→name mapping via [`DemultiplexChain`] and
/// enumerating each step's declared outputs with
/// [`enumerate_declaration_outputs`] — so demultiplexed, multi-output and
/// singleton (Progress/Inspect) steps are all reported correctly.
///
/// Chunked outputs are listed by their first chunk (`.0`, matching
/// [`ChunkPaths::nth`](fastqrab_io::io::output::chunked_writer::ChunkPaths::nth));
/// the number of further chunks is data-dependent, so they are not enumerated
/// (see [`OutputFilesListing::any_chunked`]).
pub fn list_config_output_files(toml_file: &Path) -> Result<OutputFilesListing> {
    let raw_config = crate::cli::read_config_raw(toml_file)?;
    let result = Config::tpd_from_toml(&raw_config, FieldMatchMode::AnyCase, VecMode::SingleOk);
    let checked = match result {
        Ok(config) => config,
        Err(e) => {
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

    // No [output] section -> no output files (e.g. benchmark configs).
    let Some(output) = checked.output.as_ref() else {
        return Ok(OutputFilesListing {
            files: Vec::new(),
            any_chunked: false,
        });
    };
    let prefix = output.prefix.as_str();
    let ix_sep = checked.get_ix_separator();

    let mut chain = DemultiplexChain::new();
    let mut files = Vec::new();
    let mut any_chunked = false;
    for (index, stage) in checked.stages.iter().enumerate() {
        // A step sees the demultiplex info from every Demultiplex step before it.
        if let Some(declarations) = &stage.output_declarations {
            let demultiplex_info = chain.current_info();
            for decl in declarations {
                // A chunked output's first file carries a `.0` chunk infix; the
                // remaining chunk count is data-dependent (see ChunkPaths::nth).
                let chunk_infix = if decl.chunk_policy.records_per_chunk.is_some() {
                    any_chunked = true;
                    ".0"
                } else {
                    ""
                };
                for (_tag, resolved) in enumerate_declaration_outputs(
                    decl.singleton,
                    &decl.target,
                    demultiplex_info,
                    prefix,
                    &ix_sep,
                ) {
                    files.push(match resolved {
                        ResolvedOutputName::Stdout => "--stdout--".to_string(),
                        ResolvedOutputName::File { basename, suffix } => {
                            if suffix.is_empty() {
                                format!("{basename}{chunk_infix}")
                            } else {
                                format!("{basename}{chunk_infix}.{suffix}")
                            }
                        }
                    });
                }
            }
        }
        // Advance the chain *after* this step's own outputs are enumerated, so a
        // Demultiplex step's split only affects steps downstream of it.
        if let Transformation::Demultiplex(d) = &stage.transformation {
            chain.push(index, d.declared_barcodes(), d.in_label.to_string(), &ix_sep)?;
        }
    }

    Ok(OutputFilesListing { files, any_chunked })
}
