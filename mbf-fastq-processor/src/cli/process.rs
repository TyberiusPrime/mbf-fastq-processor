use anyhow::{Context, Result, bail};
use std::path::Path;

use crate::config::CheckedConfig;
use crate::config::Config;
use crate::config::deser::{ErrorCollectorExt, FromToml};
use crate::output::OutputRunMarker;
use crate::pipeline;
use toml_edit::Document;

pub fn run(toml_file: &Path, output_directory: &Path, allow_overwrite: bool) -> Result<()> {
    let output_directory = output_directory.to_owned();
    let raw_config = ex::fs::read_to_string(toml_file)
        .with_context(|| format!("Could not read toml file: {}", toml_file.to_string_lossy()))?;
    let tomled = raw_config
        .parse::<Document<String>>()
        .context("Failed to parse TOML syntax.")?;
    let mut collector = crate::config::deser::new_error_collector();
    let parsed = Config::from_toml(tomled.as_item(), &mut collector)
        .map_err(|_| anyhow::anyhow!(collector.render(&raw_config, &toml_file.display().to_string())));
    //let parsed: anyhow::Result<_> = parsed.map_err(|e| anyhow::anyhow!("{:?}", e));
    let parsed: Config = parsed.context("Could not understand TOML file.")?;
    let checked = parsed.check()?;
    let marker_prefix = checked
        .output
        .as_ref()
        .expect("config.check() ensures output is present")
        .prefix
        .clone();
    let marker = OutputRunMarker::create(&output_directory, &marker_prefix)?;
    let allow_overwrite = allow_overwrite || marker.was_preexisting();

    let res = _run(
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
            if format!("{:?}", e).contains("already exists") {
                marker.mark_complete()?;
            }
            Err(e)
        }
    }
}

fn _run(
    mut parsed: CheckedConfig,
    output_directory: &Path,
    allow_overwrite: bool,
    raw_config: String,
) -> Result<()> {
    let start_time = std::time::Instant::now();
    let is_benchmark = parsed.benchmark.as_ref().is_some_and(|b| b.enable);
    {
        let run = pipeline::RunStage0::new(&parsed);
        let run = run.configure_demultiplex_and_init_stages(
            &mut parsed,
            &output_directory,
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
