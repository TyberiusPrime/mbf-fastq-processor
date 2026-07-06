use anyhow::{Result, bail};
use std::path::Path;
use toml_pretty_deser::prelude::*;

use crate::cli::improve_error_messages;
use crate::config::CheckedConfig;
use crate::config::Config;
use crate::output::OutputRunMarker;
use crate::pipeline;

/// # Panics
/// When there are bugs in verification
pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = crate::cli::read_config_raw(toml_file)?;
    let result = Config::tpd_from_toml(&raw_config, FieldMatchMode::AnyCase, VecMode::SingleOk);
    let parsed = match result {
        Ok(config) => config,
        Err(e) => {
            let pretty = e.pretty("config.toml");
            #[expect(
                clippy::dbg_macro,
                reason = "Used explicitly to get debug information in case of failure in TPD"
            )]
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
    let mut checked = parsed.check(toml_file == Path::new("-"))?;
    checked.raw_config = std::sync::Arc::from(raw_config.as_str());
    let (allow_overwrite, marker) = if let Some(output) = checked.output.as_ref() {
        let marker_prefix = output.prefix.clone();
        let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
        (allow_overwrite || marker.was_preexisting(), Some(marker))
    } else {
        assert!(
            checked.benchmark.is_some(),
            "No output -> expected benchmark"
        );
        (false, None)
    };

    let res = inner_run(checked, output_directory.as_ref(), allow_overwrite);

    if let Some(marker) = marker {
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
    } else {
        res
    }
}

use crate::bam_merge::MergeConfig;

fn inner_run(
    mut parsed: CheckedConfig,
    output_directory: &Path,
    allow_overwrite: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let is_benchmark = parsed.benchmark.as_ref().is_some_and(|b| b.enable);
    // Extract merge config before parsed is shadowed/moved. Merge is configured
    // on the OutputBAM step now; prefix and ix_separator stay global ([output]).
    let merge_config = parsed.output.as_ref().and_then(|output_config| {
        let separator = output_config.ix_separator.as_str();
        parsed.stages.iter().find_map(|stage| {
            let fastqrab_steps::transformations::Transformation::OutputBAM(step) =
                &stage.transformation
            else {
                return None;
            };
            let info = step.merge_info()?;
            let tails: Vec<String> = info
                .segment_names
                .iter()
                .map(|name| format!("{separator}{name}.{}", info.suffix))
                .collect();
            Some(MergeConfig {
                prefix: output_config.prefix.clone(),
                ix_separator: separator.to_string(),
                reference_label: info.reference_label,
                index_merged: info.index_merged,
                segment_tails: tails,
                records_per_molecule: info.records_per_molecule,
            })
        })
    });
    {
        let run = pipeline::RunStage0::configure_demultiplex_and_init_stages(
            &mut parsed,
            output_directory,
            allow_overwrite,
        )?;
        let run = run.create_input_threads(&parsed)?;
        let run = run.create_stage_threads(&mut parsed);
        let parsed = parsed;
        let run = run.create_output_threads(merge_config.as_ref())?;
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
