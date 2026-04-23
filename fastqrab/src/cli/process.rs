use anyhow::{Result, bail};
use std::path::Path;
use toml_pretty_deser::prelude::*;

use crate::cli::improve_error_messages;
use crate::config::CheckedConfig;
use crate::config::Config;
use crate::output::OutputRunMarker;
use crate::pipeline;

pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = crate::cli::read_config_raw(toml_file)?;
    let result = Config::tpd_from_toml(&raw_config, FieldMatchMode::AnyCase, VecMode::SingleOk);
    let parsed = match result {
        Ok(config) => config,
        Err(e) => {
            let pretty = e.pretty("config.toml");
            if pretty.trim().is_empty() {
                // shouldn't happen, but if it does, we got this error
                // cov:excl-start
                dbg!(&e);
                bail!("Failed to parse config.toml and no pretty error available");
                // cov:excl-stop
            }
            let pretty = improve_error_messages("config.toml", e);
            bail!("{pretty}");
        }
    };
    let checked = parsed.check()?;
    if toml_file == Path::new("-") && crate::cli::config_uses_stdin_fastq(&checked.input.structured)
    {
        bail!(
            "Cannot read configuration from stdin ('-') when the configuration also uses stdin \
             ('{}') for FASTQ input. Use a config file on disk instead.",
            fastqrab_io::STDIN_MAGIC_PATH
        );
    }
    let marker_prefix = checked
        .output
        .as_ref()
        .expect("config.check() ensures output is present")
        .prefix
        .clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.was_preexisting();

    let res = inner_run(
        checked,
        output_directory.as_ref(),
        allow_overwrite,
        raw_config,
    );

    match res {
        Ok(()) => {
            marker.mark_complete()?;
            Ok(())
        }
        Err(e) => {
            if format!("{e:?}").contains("already exists") {
                marker.mark_complete()?;
            }
            Err(e)
        }
    }
}

fn inner_run(
    mut parsed: CheckedConfig,
    output_directory: &Path,
    allow_overwrite: bool,
    raw_config: String,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let is_benchmark = parsed.benchmark.as_ref().is_some_and(|b| b.enable);
    // Extract merge config before parsed is shadowed/moved.
    let merge_config = parsed.output.as_ref().and_then(|o| {
        o.bam.as_ref().and_then(|b| {
            b.merge_demultiplexed.as_ref().map(|label| {
                let suffix = o.get_suffix();
                let sep = &o.ix_separator;
                // Compute segment tails from config rather than filesystem.
                // Each tail is the part of the filename that follows the combined barcode name,
                // e.g. "_read1.bam" or "_interleaved.bam".
                let segment_order = parsed.input.get_segment_order();
                let mut tails: Vec<String> = Vec::new();
                if o.interleave.is_some() {
                    tails.push(format!("{sep}interleaved.{suffix}"));
                }
                let active: Vec<&String> = match &o.output {
                    Some(list) => segment_order
                        .iter()
                        .filter(|n| list.iter().any(|l| l == *n))
                        .collect(),
                    None => segment_order.iter().collect(),
                };
                for seg in active {
                    tails.push(format!("{sep}{seg}.{suffix}"));
                }
                (
                    o.prefix.clone(),
                    o.ix_separator.clone(),
                    label.to_string(),
                    b.index_merged.unwrap_or(true),
                    tails,
                )
            })
        })
    });
    {
        let run = pipeline::RunStage0::new(&parsed);
        let run = run.configure_demultiplex_and_init_stages(
            &mut parsed,
            output_directory,
            allow_overwrite,
        )?;
        let run = run.create_input_threads(&parsed)?;
        let run = run.create_stage_threads(&mut parsed);
        let parsed = parsed;
        let run = run.create_output_threads(&parsed, raw_config)?;
        let run = run.join_threads();

        let errors = run.errors;

        if !errors.is_empty() {
            bail!(errors.join("\n"));
        }

        if let Some((prefix, sep, merge_label, index_merged, segment_tails)) = merge_config {
            crate::bam_merge::merge_demultiplexed_bam(
                output_directory,
                &prefix,
                &sep,
                &run.demultiplex_infos,
                &run.demultiplex_step_infos,
                &merge_label,
                &segment_tails,
                index_merged,
                &run.reads_per_tag,
            )?;
        }

        drop(parsed);
    }
    if is_benchmark {
        let elapsed = start_time.elapsed();
        println!(
            "Benchmark completed in {:.2?} seconds",
            elapsed.as_secs_f64()
        );
    }
    Ok(())
}
