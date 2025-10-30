#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

use anyhow::{Context, Result};
use config::Config;
use output::OutputRunMarker;
use std::path::Path;
use transformations::Transformation;

pub mod config;
pub mod demultiplex;
mod dna;
pub mod documentation;
pub mod io;
mod output;
mod pipeline;
mod transformations;

pub use io::FastQRead;

use crate::demultiplex::Demultiplexed;

#[allow(clippy::similar_names)] // I like rx/tx nomenclature
#[allow(clippy::too_many_lines)] //todo: this is true.
pub fn run(
    toml_file: &Path,
    output_directory: &Path, //todo: figure out wether this is just an output directory, or a
                             //*working* directory
) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let mut parsed = eserde::toml::from_str::<Config>(&raw_config)
        .with_context(|| format!("Could not parse toml file: {}", toml_file.to_string_lossy()))?;
    parsed.check().context("Error in configuration")?;
    let (mut parsed, report_labels) = Transformation::expand(parsed);
    let marker_prefix = parsed.output.as_ref().unwrap().prefix.clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = marker.preexisting();
    //parsed.transform = new_transforms;
    //let start_time = std::time::Instant::now();
    #[allow(clippy::if_not_else)]
    {
        let run = pipeline::RunStage0::new(&parsed);
        let run = run.configure_demultiplex_and_init_stages(
            &mut parsed,
            &output_directory,
            allow_overwrite,
        )?;
        let run = run.create_input_threads(&parsed)?;
        let run = run.create_stage_threads(&mut parsed);
        let parsed = parsed; //after this, stages are transformed and ready, and config is read only.
        let run = run.create_output_threads(&parsed, report_labels, raw_config)?;
        let run = run.join_threads();
        //
        //promote all panics to actual process failures with exit code != 0
        let errors = run.errors;
        if !errors.is_empty() {
            eprintln!("\nErrors occurred during processing:");
            for error in &errors {
                eprintln!("{error}");
            }
            std::process::exit(101);
        }
        //assert!(errors.is_empty(), "Error in threads occured: {errors:?}");

        //ok all this needs is a buffer that makes sure we reorder correctly at the end.
        //and then something block based, not single reads to pass between the threads.
        drop(parsed);
    }

    marker.mark_complete()?;
    Ok(())
}

pub(crate) fn join_nonempty<'a>(
    parts: impl IntoIterator<Item = &'a str>,
    separator: &str,
) -> String {
    let mut iter = parts.into_iter().filter(|part| !part.is_empty());
    let mut result = String::new();
    if let Some(first) = iter.next() {
        result.push_str(first);
        for part in iter {
            result.push_str(separator);
            result.push_str(part);
        }
    }
    result
}
