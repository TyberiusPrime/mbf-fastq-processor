use allocation_counter::measure;
use clap::{Arg, ArgAction, Command};
use human_panic::{Metadata, setup_panic};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{Context, Result};

fn build_cli() -> Command {
    // Construct version string with git commit hash
    // Using const_format doesn't work here due to option_env, so we leak the string
    let version_string: &'static str = Box::leak(
        format!(
            "{} (git: {})",
            env!("CARGO_PKG_VERSION"),
            option_env!("COMMIT_HASH").unwrap_or("unknown")
        )
        .into_boxed_str(),
    );

    Command::new("mbf-fastq-processor")
        .version(version_string)
        .about(
            "Process FASTQ files with filtering, sampling, slicing, demultiplexing, and analysis",
        )
        .after_help(
            "Minimal configuration.toml:\n\n\
            [input]\n\
                read_1 = 'input_R1.fq.gz'\n\n\
            [[step]]\n\
                action = 'Report'\n\
                label = 'my_report'\n\
                count = true\n\n\
            [output]\n\
                prefix = 'output'\n\
                report_html = true\n",
        )
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("process")
                .about("Process FASTQ files using a configuration file")
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file")
                        .required(true)
                        .value_name("CONFIG_TOML"),
                )
                .arg(
                    Arg::new("output_dir")
                        .help("Output directory (deprecated, for backward compatibility)")
                        .value_name("OUTPUT_DIR")
                        .hide(true),
                )
                .arg(
                    Arg::new("allow-overwrite")
                        .long("allow-overwrite")
                        .help("Allow overwriting existing output files")
                        .action(ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("template")
                .about("Output configuration template or subsection")
                .arg(
                    Arg::new("section")
                        .help("Optional section name to output template for")
                        .value_name("SECTION"),
                ),
        )
        .subcommand(
            Command::new("cookbook")
                .about("List available cookbooks or show a specific cookbook")
                .arg(
                    Arg::new("number")
                        .help("Cookbook number to display")
                        .value_name("NUMBER"),
                ),
        )
        .subcommand(Command::new("list-steps").about("List all available transformation steps"))
        .subcommand(Command::new("version").about("Output version information"))
        .subcommand(
            Command::new("validate")
                .about("Validate a configuration file without processing")
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file to validate")
                        .required(true)
                        .value_name("CONFIG_TOML"),
                ),
        )
        .subcommand(
            Command::new("interactive")
                .about("Interactive mode: watch a TOML file and show live results")
                .long_about(
                    "Interactive mode continuously watches a TOML configuration file for changes. \
                    When the file changes, it automatically:\n\
                    - Prepends Head and FilterReservoirSample steps to limit processing\n\
                    - Appends an Inspect step to show sample results\n\
                    - Adjusts paths and output for quick testing\n\
                    - Displays results or errors in a pretty format\n\n\
                    This is ideal for rapid development and testing of processing pipelines."
                )
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file to watch")
                        .required(true)
                        .value_name("CONFIG_TOML"),
                ),
        )
}

fn print_template(step: Option<&String>) {
    print!(
        "{}",
        mbf_fastq_processor::documentation::get_template(step.map(String::as_str))
            .unwrap_or(std::borrow::Cow::Borrowed("No such documentation found"))
    );
}

fn comment(text: &str) -> String {
    let mut commented = String::new();
    for line in text.lines() {
        commented.push_str("# ");
        commented.push_str(line);
        commented.push('\n');
    }
    commented
}

fn print_cookbook(cookbook_number: Option<&String>) {
    match cookbook_number {
        None => {
            // List all cookbooks
            println!("Available cookbooks:\n");
            let cookbooks = mbf_fastq_processor::cookbooks::list_cookbooks();
            for (number, name) in cookbooks {
                println!("  {number}. {name}");
            }
            println!("\nUse 'cookbook <number>' to view a specific cookbook.");
        }
        Some(num_str) => {
            // Show specific cookbook
            match num_str.parse::<usize>() {
                Ok(num) => {
                    if let Some(cookbook) = mbf_fastq_processor::cookbooks::get_cookbook(num) {
                        println!("{}", comment(cookbook.readme));
                        println!("\n## Configuration (input.toml)\n");
                        println!("{}", cookbook.toml);
                    } else {
                        eprintln!("Error: Cookbook {} not found", num);
                        eprintln!(
                            "Available cookbooks: 1-{}",
                            mbf_fastq_processor::cookbooks::cookbook_count()
                        );
                        std::process::exit(1);
                    }
                }
                Err(_) => {
                    eprintln!("Error: Invalid cookbook number '{}'", num_str);
                    std::process::exit(1);
                }
            }
        }
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn main() -> Result<()> {
    if std::env::var("NO_FRIENDLY_PANIC").is_err() && std::env::var("RUST_BACKTRACE").is_err() {
        setup_panic!(
        Metadata::new(
            env!("CARGO_PKG_NAME"),
            format!(
                "{} (git: {})",
                env!("CARGO_PKG_VERSION"),
                option_env!("COMMIT_HASH").unwrap_or("unknown")
            )
        )
            //.authors("My Company Support <support@mycompany.com>")
            .homepage("https://github.com/TyberiusPrime/mbf-fastq-processor")
            .support("Open a github issue at https://github.com/TyberiusPrime/mbf-fastq-processor/issues/new and attach the crash report.")
    );
    }

    assert!(
        !std::env::args().any(|x| x == "--test-friendly-panic"),
        "friendly panic test!"
    );

    // Check for backward compatibility: direct .toml file path as first argument
    if let Some(first_arg) = std::env::args().nth(1) {
        if first_arg.ends_with(".toml") && !first_arg.starts_with('-') {
            // Old-style invocation: direct toml file path
            run_with_optional_measure(|| process_from_toml_file(&first_arg, false));
            return Ok(());
        }
    }

    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("process", sub_matches)) => {
            let config_file = sub_matches
                .get_one::<String>("config")
                .context("Config file argument is required")?;
            let allow_overwrites = sub_matches.get_flag("allow-overwrite");
            run_with_optional_measure(|| process_from_toml_file(config_file, allow_overwrites));
        }
        Some(("template", sub_matches)) => {
            let section = sub_matches.get_one::<String>("section");
            print_template(section);
            std::process::exit(0);
        }
        Some(("cookbook", sub_matches)) => {
            let number = sub_matches.get_one::<String>("number");
            print_cookbook(number);
            std::process::exit(0);
        }
        Some(("list-steps", _)) => {
            print!("{}", mbf_fastq_processor::list_steps::format_steps_list());
            std::process::exit(0);
        }
        Some(("version", _)) => {
            print_version_and_exit();
        }
        Some(("validate", sub_matches)) => {
            let config_file = sub_matches
                .get_one::<String>("config")
                .context("Config file argument is required")?;
            validate_config_file(config_file);
        }
        Some(("interactive", sub_matches)) => {
            let config_file = sub_matches
                .get_one::<String>("config")
                .context("Config file argument is required")?;
            run_interactive_mode(config_file);
        }
        _ => {
            // This shouldn't happen due to arg_required_else_help, but just in case
            build_cli().print_help()?;
            std::process::exit(1);
        }
    }
    Ok(())
}

fn print_version_and_exit() {
    println!(
        "{} (git: {})",
        env!("CARGO_PKG_VERSION"),
        option_env!("COMMIT_HASH").unwrap_or("unknown")
    );
    std::process::exit(0);
}

fn docs_matching_error_message(e: &anyhow::Error) -> String {
    use std::fmt::Write;
    let mut docs = String::new();
    let str_error = format!("{e:?}");
    let re = regex::Regex::new(r"[(]([^)]+)[)]").unwrap();
    let mut seen = HashSet::new();
    for cap in re.captures_iter(&str_error) {
        let step = &cap[1];
        let template = mbf_fastq_processor::documentation::get_template(Some(step));
        if !seen.insert(template.clone()) {
            continue;
        }
        if let Some(template) = template {
            write!(docs, "\n\n ==== {step} ====:\n{template}\n").unwrap();
        }
    }
    docs
}

/// We can't fight all aliases, but at least remove those that are
/// just capitalization variants of each other.
/// Prefer the ones with more capital letters
fn canonicalize_variants(parts: Vec<&str>) -> Vec<String> {
    let mut seen: HashMap<String, String> = HashMap::new();
    for p in parts {
        let key = p.to_lowercase();
        match seen.get(&key) {
            Some(existing) => {
                if p.chars().filter(|c| c.is_uppercase()).count()
                    > existing.chars().filter(|c| c.is_uppercase()).count()
                {
                    seen.insert(key, p.to_string());
                }
            }
            None => {
                seen.insert(key, p.to_string());
            }
        }
    }
    seen.into_values().collect()
}

/// Formats error messages by adding some newlines and indention for readability
fn prettyify_error_message(error: &str) -> String {
    let lines: Vec<&str> = error.lines().collect();
    let mut formatted_lines = Vec::new();

    let regex = Regex::new(r"([^:]+: )unknown variant `([^`]+)`, expected one of (.+)").unwrap();

    for line in lines {
        if line == "    in `action`" {
            continue;
        }
        if let Some(matches) = regex.captures(line) {
            let prefix = &matches[1];
            let unknown_variant = &matches[2];
            let expected_variants = &matches[3];

            let parts: Vec<&str> = expected_variants
                .split(", ")
                .filter(|x| !x.starts_with("`_"))
                .map(|s| s.trim_matches('`'))
                .collect();

            let parts = canonicalize_variants(parts);

            if parts.len() > 1 {
                let formatted_suffix = parts.join(",\n\t");
                let mut parts = parts; // so on equal distance, we have alphabetical order
                parts.sort();
                let mut levenstein_distances = parts
                    .into_iter()
                    .map(|part| {
                        let dist = bio::alignment::distance::levenshtein(
                            unknown_variant.as_bytes(),
                            part.as_bytes(),
                        );

                        (part, dist)
                    })
                    .collect::<Vec<(String, u32)>>();
                levenstein_distances.sort_by_key(|&(_, dist)| dist);
                let best_three = levenstein_distances
                    .iter()
                    .take(3)
                    .map(|(part, _)| format!("`{part}`"))
                    .collect::<Vec<String>>()
                    .join(", ");
                let msg = format!(
                    "{prefix}Unknown variant `{unknown_variant}`. Did you mean one of {best_three}?"
                );
                formatted_lines.push(msg);
                if prefix.ends_with(".action: ") {
                    formatted_lines.push(
                        "\tTo list available steps, run the `list-steps` command".to_string(),
                    );
                } else {
                    formatted_lines.push(format!("Available: \n\t{formatted_suffix}"));
                }
            } else {
                formatted_lines.push(line.to_string());
            }
        } else {
            formatted_lines.push(line.to_string());
        }
    }

    formatted_lines.join("\n")
}

fn process_from_toml_file(toml_file: &str, allow_overwrites: bool) {
    let toml_file = PathBuf::from(toml_file);
    let current_dir = std::env::current_dir().unwrap();
    if let Err(e) = mbf_fastq_processor::run(&toml_file, &current_dir, allow_overwrites) {
        eprintln!("Unfortunatly an error was detected and lead to an early exit.\n");
        let docs = docs_matching_error_message(&e);
        if !docs.is_empty() {
            let indented_docs = docs
                .trim()
                .lines()
                .map(|line| format!("    {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            eprintln!("# == Documentation == \n(from the 'template' command)\n{indented_docs}\n",);
        }

        eprintln!(
            "# == Error Details ==\n{}",
            prettyify_error_message(&format!("{e:?}"))
        );
        std::process::exit(1);
    }
}

fn validate_config_file(toml_file: &str) {
    let toml_file = PathBuf::from(toml_file);
    match mbf_fastq_processor::validate_config(&toml_file) {
        Ok(warnings) => {
            if warnings.is_empty() {
                println!("✓ Configuration is valid");
                std::process::exit(0);
            } else {
                println!("✓ Configuration is valid (with warnings)");
                for warning in warnings {
                    eprintln!("Warning: {warning}");
                }
                std::process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Configuration validation failed:\n");
            let docs = docs_matching_error_message(&e);
            if !docs.is_empty() {
                let indented_docs = docs
                    .trim()
                    .lines()
                    .map(|line| format!("    {line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                eprintln!(
                    "# == Documentation == \n(from the 'template' command)\n{indented_docs}\n",
                );
            }

            eprintln!(
                "# == Error Details ==\n{}",
                prettyify_error_message(&format!("{e:?}"))
            );
            std::process::exit(1);
        }
    }
}

fn run_interactive_mode(toml_file: &str) {
    let toml_file = PathBuf::from(toml_file);
    if let Err(e) = mbf_fastq_processor::interactive::run_interactive(&toml_file) {
        eprintln!("Interactive mode error: {:?}", e);
        std::process::exit(1);
    }
}

fn run_with_optional_measure<F>(f: F)
where
    F: FnOnce(),
{
    if std::env::var("RUST_MEASURE_ALLOC")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        let info = measure(f);
        eprintln!(
            "alloc: count_total={} count_max={} count_current={} bytes_total={} bytes_max={} bytes_current={}",
            info.count_total,
            info.count_max,
            info.count_current,
            info.bytes_total,
            info.bytes_max,
            info.bytes_current
        );
    } else {
        f();
    }
}
