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
            #[expect(clippy::dbg_macro, reason="Used explicitly to get debug information in case of failure in TPD")]
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

use crate::bam_merge::MergeConfig;

fn inner_run(
    mut parsed: CheckedConfig,
    output_directory: &Path,
    allow_overwrite: bool,
    raw_config: String,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let is_benchmark = parsed.benchmark.as_ref().is_some_and(|b| b.enable);
    // Extract merge config before parsed is shadowed/moved.
    let merge_config = if let Some(output_config) = parsed.output.as_ref()
        && let Some(bam_output_config) = output_config.bam.as_ref()
        && bam_output_config
            .merge_demultiplexed
            .as_ref()
            .is_some_and(|m| *m)
        && let Some(reference_tag) = bam_output_config.tag_to_reference.as_ref().map(|x| &x.tag)
    {
        let suffix = output_config.get_suffix();
        let sep = output_config.ix_separator.as_str();
        let mut tails: Vec<String> = Vec::new();
        if output_config.interleave.is_some() {
            tails.push(format!("{sep}interleaved.{suffix}"));
        } else {
            let active = output_config.output.as_ref().expect("parent was ok");
            for seg in active {
                tails.push(format!("{sep}{seg}.{suffix}"));
            }
        }
        Some(MergeConfig {
            prefix: output_config.prefix.to_string(),
            ix_separator: sep.to_string(),
            reference_label: reference_tag.clone(),
            index_merged: bam_output_config.index_merged,
            segment_tails: tails,
        })
    } else {
        None
    };
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
        let run = run.create_output_threads(&parsed, raw_config, merge_config.as_ref())?;
        let run = run.join_threads();

        let errors = run.errors;

        if !errors.is_empty() {
            bail!(errors.join("\n"));
        }

        if let Some(merge_config) = merge_config {
            let handles = run
                .merge_bam_handles
                .expect("merge_bam_handles must be Some when merge_config is Some");
            crate::bam_merge::merge_demultiplexed_bam(
                output_directory,
                &merge_config.prefix,
                &merge_config.ix_separator,
                &run.demultiplex_infos,
                &run.demultiplex_step_infos,
                &merge_config.reference_label,
                &merge_config.segment_tails,
                merge_config.index_merged,
                &run.reads_per_tag,
                handles,
            )?; // cov:excl-line
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
