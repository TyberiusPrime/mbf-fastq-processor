use allocation_counter::measure;
use human_panic::{Metadata, setup_panic};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{Context, Result, bail};

fn get_version_string() -> String {
    format!(
        "{} (git: {})",
        env!("CARGO_PKG_VERSION"),
        option_env!("COMMIT_HASH").unwrap_or("unknown")
    )
}

fn print_help() {
    println!("mbf-fastq-processor {}", get_version_string());
    println!("Process FASTQ files with filtering, sampling, slicing, demultiplexing, and analysis");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor <SUBCOMMAND>");
    println!();
    println!("Subcommands:");
    println!("    process        Process FASTQ files using a configuration file");
    println!("    template       Output configuration template or subsection");
    println!("    cookbook       List available cookbooks or show a specific cookbook");
    println!("    list-steps     List all available transformation steps");
    println!("    version        Output version information");
    println!("    validate       Validate a configuration file without processing");
    println!("    interactive    Interactive mode: watch a TOML file and show live results");
    println!("    help           Print this message");
    println!();
    println!("Quick start:");
    println!("    1. mbf-fastq-processor cookbook # pick one");
    println!("    2. mbf-fastq-processor cookbook <no from 1> > pipeline.toml");
    println!("    3. Edit pipeline.toml with your input files");
    println!("    4. mbf-fastq-processor process pipeline.toml");
    println!();
    println!("Docs:");
    println!("    Visit https://tyberiusprime.github.io/mbf-fastq-processor/ for in depth documentation");
    println!("    following the Diátaxis framework.");
}

fn print_process_help() {
    println!("Process FASTQ files using a configuration file");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor process [OPTIONS] [CONFIG_TOML]");
    println!();
    println!("Args:");
    println!("    <CONFIG_TOML>    Path to the TOML configuration file (optional, auto-detected if not specified)");
    println!();
    println!("Options:");
    println!("    --allow-overwrite    Allow overwriting existing output files");
    println!("    -h, --help           Print help information");
}

fn print_template_help() {
    println!("Output configuration template or subsection");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor template [SECTION]");
    println!();
    println!("Args:");
    println!("    <SECTION>    Optional section name to output template for");
    println!();
    println!("Options:");
    println!("    -h, --help    Print help information");
}

fn print_cookbook_help() {
    println!("List available cookbooks or show a specific cookbook");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor cookbook [NUMBER]");
    println!();
    println!("Args:");
    println!("    <NUMBER>    Cookbook number to display");
    println!();
    println!("Options:");
    println!("    -h, --help    Print help information");
}

fn print_validate_help() {
    println!("Validate a configuration file without processing");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor validate <CONFIG_TOML>");
    println!();
    println!("Args:");
    println!("    <CONFIG_TOML>    Path to the TOML configuration file to validate");
    println!();
    println!("Options:");
    println!("    -h, --help    Print help information");
}

fn print_interactive_help() {
    println!("Interactive mode: watch a TOML file and show live results");
    println!();
    println!("Interactive mode continuously watches a TOML configuration file for changes.");
    println!("When the file changes, it automatically:");
    println!("  - Prepends Head and FilterReservoirSample steps to limit processing");
    println!("  - Appends an Inspect step to show sample results");
    println!("  - Adjusts paths and output for quick testing");
    println!("  - Displays results or errors in a pretty format");
    println!();
    println!("This is ideal for rapid development and testing of processing pipelines.");
    println!();
    println!("If no config file is specified, will auto-detect a single .toml file in the");
    println!("current directory that contains both [input] and [output] sections.");
    println!();
    println!("Usage:");
    println!("    mbf-fastq-processor interactive [OPTIONS] [CONFIG_TOML]");
    println!();
    println!("Args:");
    println!("    <CONFIG_TOML>    Path to the TOML configuration file to watch");
    println!("                     (optional if only one valid .toml in current directory)");
    println!();
    println!("Options:");
    println!("    -n, --head <N>       Number of reads to process (default: 10000)");
    println!("    -s, --sample <N>     Number of reads to sample for display (default: 15)");
    println!("    -i, --inspect <N>    Number of reads to display in inspect output (default: 15)");
    println!("    -h, --help           Print help information");
}

#[derive(Debug)]
enum Subcommand {
    Process {
        config: Option<String>,
        allow_overwrite: bool,
    },
    Template {
        section: Option<String>,
    },
    Cookbook {
        number: Option<String>,
    },
    ListSteps,
    Version,
    Validate {
        config: String,
    },
    Interactive {
        config: Option<String>,
        head: Option<u64>,
        sample: Option<u64>,
        inspect: Option<u64>,
    },
    Help,
}

fn parse_args() -> Result<Subcommand> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();

    // Get the subcommand
    let subcommand = match parser.next()? {
        Some(Value(cmd)) => cmd.string()?,
        Some(Long("help") | Short('h')) => return Ok(Subcommand::Help),
        Some(Long("version") | Short('V')) => return Ok(Subcommand::Version),
        None => {
            // No arguments provided - print help and exit
            print_help();
            std::process::exit(2);
        }
        _ => bail!("Unexpected argument"),
    };

    match subcommand.as_str() {
        "process" => {
            let mut config = None;
            let mut allow_overwrite = false;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("allow-overwrite") => allow_overwrite = true,
                    Long("help") | Short('h') => {
                        print_process_help();
                        std::process::exit(0);
                    }
                    Value(val) => {
                        if config.is_none() {
                            config = Some(val.string()?);
                        } else {
                            // Second positional arg (deprecated output_dir), ignore it
                        }
                    }
                    _ => bail!("Unexpected argument for process: {:?}", arg),
                }
            }

            Ok(Subcommand::Process { config, allow_overwrite })
        }
        "template" => {
            let mut section = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_template_help();
                        std::process::exit(0);
                    }
                    Value(val) => {
                        section = Some(val.string()?);
                    }
                    _ => bail!("Unexpected argument for template: {:?}", arg),
                }
            }

            Ok(Subcommand::Template { section })
        }
        "cookbook" => {
            let mut number = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_cookbook_help();
                        std::process::exit(0);
                    }
                    Value(val) => {
                        number = Some(val.string()?);
                    }
                    _ => bail!("Unexpected argument for cookbook: {:?}", arg),
                }
            }

            Ok(Subcommand::Cookbook { number })
        }
        "list-steps" => {
            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        println!("List all available transformation steps");
                        println!();
                        println!("Usage:");
                        println!("    mbf-fastq-processor list-steps");
                        std::process::exit(0);
                    }
                    _ => bail!("list-steps takes no arguments"),
                }
            }
            Ok(Subcommand::ListSteps)
        }
        "version" => {
            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        println!("Output version information");
                        println!();
                        println!("Usage:");
                        println!("    mbf-fastq-processor version");
                        std::process::exit(0);
                    }
                    _ => bail!("version takes no arguments"),
                }
            }
            Ok(Subcommand::Version)
        }
        "validate" => {
            let mut config = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_validate_help();
                        std::process::exit(0);
                    }
                    Value(val) => {
                        config = Some(val.string()?);
                    }
                    _ => bail!("Unexpected argument for validate: {:?}", arg),
                }
            }

            let config = config.context("validate requires a config file argument")?;
            Ok(Subcommand::Validate { config })
        }
        "interactive" => {
            let mut config = None;
            let mut head = None;
            let mut sample = None;
            let mut inspect = None;

            while let Some(arg) = parser.next()? {
                match arg {
                    Long("help") | Short('h') => {
                        print_interactive_help();
                        std::process::exit(0);
                    }
                    Long("head") | Short('n') => {
                        head = Some(parser.value()?.string()?.parse()?);
                    }
                    Long("sample") | Short('s') => {
                        sample = Some(parser.value()?.string()?.parse()?);
                    }
                    Long("inspect") | Short('i') => {
                        inspect = Some(parser.value()?.string()?.parse()?);
                    }
                    Value(val) => {
                        config = Some(val.string()?);
                    }
                    _ => bail!("Unexpected argument for interactive: {:?}", arg),
                }
            }

            Ok(Subcommand::Interactive { config, head, sample, inspect })
        }
        "help" => Ok(Subcommand::Help),
        _ => bail!("error: unrecognized subcommand '{}'", subcommand),
    }
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
            run_with_optional_measure(|| process_from_toml_file(&first_arg.into(), false));
            return Ok(());
        }
    }

    let subcommand = match parse_args() {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    };

    match subcommand {
        Subcommand::Process { config, allow_overwrite } => {
            // Auto-discover TOML file if not specified
            let toml_path = match config {
                Some(path) => PathBuf::from(path),
                None => match find_single_valid_toml() {
                    Ok(path) => {
                        println!("Auto-detected configuration file: {}", path.display());
                        path
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        eprintln!(
                            "\nPlease specify a configuration file explicitly: \
                     mbf-fastq-processor process <config.toml>"
                        );
                        std::process::exit(1);
                    }
                },
            };
            run_with_optional_measure(|| process_from_toml_file(&toml_path, allow_overwrite));
        }
        Subcommand::Template { section } => {
            print_template(section.as_ref());
            std::process::exit(0);
        }
        Subcommand::Cookbook { number } => {
            print_cookbook(number.as_ref());
            std::process::exit(0);
        }
        Subcommand::ListSteps => {
            print!("{}", mbf_fastq_processor::list_steps::format_steps_list());
            std::process::exit(0);
        }
        Subcommand::Version => {
            print_version_and_exit();
        }
        Subcommand::Validate { config } => {
            validate_config_file(&config);
        }
        Subcommand::Interactive { config, head, sample, inspect } => {
            run_interactive_mode(config.as_ref(), head, sample, inspect);
        }
        Subcommand::Help => {
            print_help();
            std::process::exit(0);
        }
    }
    Ok(())
}

fn print_version_and_exit() {
    println!("{}", get_version_string());
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

fn process_from_toml_file(toml_file: &PathBuf, allow_overwrites: bool) {
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

fn run_interactive_mode(
    toml_file: Option<&String>,
    head: Option<u64>,
    sample: Option<u64>,
    inspect: Option<u64>,
) {
    // Auto-discover TOML file if not specified
    let toml_path = match toml_file {
        Some(path) => PathBuf::from(path),
        None => match find_single_valid_toml() {
            Ok(path) => {
                println!("Auto-detected configuration file: {}", path.display());
                path
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!(
                    "\nPlease specify a configuration file explicitly: \
                     mbf-fastq-processor interactive <config.toml>"
                );
                std::process::exit(1);
            }
        },
    };

    if let Err(e) =
        mbf_fastq_processor::interactive::run_interactive(&toml_path, head, sample, inspect)
    {
        eprintln!("Interactive mode error: {:?}", e);
        std::process::exit(1);
    }
}

/// Find a single .toml file in the current directory that has both [input] and [output] sections
fn find_single_valid_toml() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let mut valid_tomls = Vec::new();

    for entry in ex::fs::read_dir(&current_dir)
        .with_context(|| format!("Failed to read directory: {}", current_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            // Try to read and parse the TOML to check for [input] and [output] sections
            if let Ok(content) = ex::fs::read_to_string(&path) {
                // Simple check: does it contain [input] and [output]?
                if content.contains("[input]") && content.contains("[output]") {
                    valid_tomls.push(path);
                }
            }
        }
    }

    match valid_tomls.len() {
        0 => bail!(
            "No valid TOML configuration files found in current directory.\n\
             A valid configuration must contain both [input] and [output] sections."
        ),
        1 => Ok(valid_tomls.into_iter().next().unwrap()),
        n => bail!(
            "Found {} valid TOML files in current directory. Please specify which one to use:\n{}",
            n,
            valid_tomls
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ),
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
