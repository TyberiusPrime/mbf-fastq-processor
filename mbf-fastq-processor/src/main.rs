use allocation_counter::measure;
use anyhow::{Context, Result, bail};
use clap::{Arg, ArgAction, Command, ValueHint, value_parser};
use clap_complete::{Generator, Shell, generate};
use human_panic::{Metadata, setup_panic};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Path, PathBuf},
};

#[allow(clippy::too_many_lines)]
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
            "Quick start:
    1. mbf-fastq-processor cookbook # pick one
    2. mbf-fastq-processor cookbook <no from 1> > pipeline.toml
    2. Edit pipeline.toml with your input files
    3. mbf-fastq-processor process pipeline.toml

Docs:
    Visit https://tyberiusprime.github.io/mbf-fastq-processor/ for in depth documentation
    following the Diátaxis framework.
",
        )
        .subcommand_required(false)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("process")
                .about("Process FASTQ files using a configuration file")
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file")
                        .required(false)
                        .value_name("CONFIG_TOML")
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("output_dir")
                        .help("Output directory (deprecated, for backward compatibility)")
                        .value_name("OUTPUT_DIR")
                        .value_hint(ValueHint::DirPath)
                        .hide(true),
                )
                .arg(
                    Arg::new("allow-overwrite")
                        .long("allow-overwrite")
                        .help("Allow overwriting existing output files")
                        .action(ArgAction::SetTrue),
                )
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
                        .required(false)
                        .value_name("CONFIG_TOML")
                        .value_hint(ValueHint::FilePath),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Run processing in a temp directory and verify outputs match expected outputs or expected panics")
                .long_about(
                    "Verifies that running a configuration produces expected outputs.\n\
For normal tests:\n\
- Runs the configuration and compares output files with expected outputs in the same directory\n\
\n\
For panic tests:\n\
- If 'expected_panic.txt' exists: expects command to fail with stderr containing the exact text\n\
- If 'expected_panic.regex' exists: expects command to fail with stderr matching the regex pattern\n\
\n\
This command is used by the test runner but can also be run manually to verify test cases."
                )
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file (optional if only one valid .toml in current directory)")
                        .required(false)
                        .value_name("CONFIG_TOML")
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("output-dir")
                        .long("output-dir")
                        .help("Directory to copy outputs to if verification fails (will be removed if exists)")
                        .value_name("OUTPUT_DIR")
                        .value_hint(ValueHint::DirPath),
                )
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
                    This is ideal for rapid development and testing of processing pipelines.\n\n\
                    If no config file is specified, will auto-detect a single .toml file in the \
                    current directory that contains both [input] and [output] sections."
                )
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file to watch (optional if only one valid .toml in current directory)")
                        .value_name("CONFIG_TOML")
                        .value_hint(ValueHint::FilePath),
                )
                .arg(
                    Arg::new("head")
                        .long("head")
                        .short('n')
                        .help("Number of reads to process (default: 10000)")
                        .value_name("N")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("sample")
                        .long("sample")
                        .short('s')
                        .help("Number of reads to sample for display (default: 15)")
                        .value_name("N")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("inspect")
                        .long("inspect")
                        .short('i')
                        .help("Number of reads to display in inspect output (default: 15)")
                        .value_name("N")
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("completions")
                .about("Generate shell completion scripts")
                .long_about(
                    "Generate shell completion scripts for various shells.\n\n\
                    Supported shells: bash, fish, zsh, powershell, elvish\n\n\
                    Installation instructions:\n\
                    • Bash:       echo 'source <(mbf-fastq-processor completions bash)' >> ~/.bashrc\n\
                    • Fish:       mbf-fastq-processor completions fish > ~/.config/fish/completions/mbf-fastq-processor.fish\n\
                    • Zsh:        echo 'source <(mbf-fastq-processor completions zsh)' >> ~/.zshrc\n\
                    • PowerShell: mbf-fastq-processor completions powershell | Out-String | Invoke-Expression"
                )
                .arg(
                    Arg::new("shell")
                        .help("Shell to generate completions for")
                        .required(true)
                        .value_parser(value_parser!(Shell))
                        .value_name("SHELL"),
                ),
        )
}

/// Generate shell completions and print to stdout
fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut io::stdout(),
    );
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
            println!("\nUse 'cookbook <number>|<name>' to view a specific cookbook.");
        }
        Some(num_str) => {
            // Show specific cookbook
            let cookbook = num_str
                .parse::<usize>()
                .ok()
                .and_then(|num| mbf_fastq_processor::cookbooks::get_cookbook(num))
                .or_else(|| mbf_fastq_processor::cookbooks::get_cookbook_by_name(num_str));
            if let Some(cookbook) = cookbook {
                println!("{}", comment(cookbook.readme));
                println!("\n## Configuration (input.toml)\n");
                println!("{}", cookbook.toml);
            } else {
                eprintln!("Error: Cookbook {num_str} not found");
                eprintln!(
                    "Available cookbooks: 1-{}",
                    mbf_fastq_processor::cookbooks::cookbook_count()
                );
                std::process::exit(1);
            }
        }
    }
}

fn handle_toml_arg(config_file: Option<&String>) -> PathBuf {
    match config_file {
        Some(path) => PathBuf::from(path),
        None => match find_single_valid_toml() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error: {e}");
                eprintln!(
                    "\nPlease specify a configuration file explicitly: \
                     mbf-fastq-processor verify <config.toml>"
                );
                std::process::exit(1);
            }
        },
    }
}

#[allow(clippy::case_sensitive_file_extension_comparisons)]
#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    // Support environment-based completion generation (modern approach)
    // Usage: COMPLETE=bash mbf-fastq-processor
    if let Ok(shell_str) = std::env::var("COMPLETE")
        && let Ok(shell) = shell_str.parse::<Shell>()
    {
        let mut cmd = build_cli();
        print_completions(shell, &mut cmd);
        return Ok(());
    }

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

    // // Check for backward compatibility: direct .toml file path as first argument
    // if let Some(first_arg) = std::env::args().nth(1) {
    //     if first_arg.ends_with(".toml") && !first_arg.starts_with('-') {
    //         // Old-style invocation: direct toml file path
    //         run_with_optional_measure(|| process_from_toml_file(&PathBuf::from(&first_arg), false, false));
    //         return Ok(());
    //     }
    // }

    let matches = build_cli().get_matches();

    match matches.subcommand() {
        Some(("process", sub_matches)) => {
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"));
            let allow_overwrites = sub_matches.get_flag("allow-overwrite");
            run_with_optional_measure(|| process_from_toml_file(&toml_path, allow_overwrites));
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
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"));
            validate_config_file(toml_path);
        }
        Some(("verify", sub_matches)) => {
            let output_dir = sub_matches.get_one::<String>("output-dir");

            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"));
            verify_config_file(&toml_path, output_dir.map(PathBuf::from));
        }
        Some(("interactive", sub_matches)) => {
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"));
            let head = sub_matches.get_one::<u64>("head").copied();
            let sample = sub_matches.get_one::<u64>("sample").copied();
            let inspect = sub_matches.get_one::<u64>("inspect").copied();
            run_interactive_mode(toml_path, head, sample, inspect);
        }
        Some(("completions", sub_matches)) => {
            if let Some(shell) = sub_matches.get_one::<Shell>("shell") {
                let mut cmd = build_cli();
                print_completions(*shell, &mut cmd);
                std::process::exit(0);
            }
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
    let re = regex::Regex::new(r"[(]([^)]+)[)]").expect("hardcoded regex pattern is valid");
    let mut seen = HashSet::new();
    for cap in re.captures_iter(&str_error) {
        let step = &cap[1];
        let template = mbf_fastq_processor::documentation::get_template(Some(step));
        if !seen.insert(template.clone()) {
            continue;
        }
        if let Some(template) = template {
            write!(docs, "\n\n ==== {step} ====:\n{template}\n")
                .expect("writing to String never fails");
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

    let regex = Regex::new(r"([^:]+: )unknown variant `([^`]+)`, expected one of (.+)")
        .expect("hardcoded regex pattern is valid");

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

fn process_from_toml_file(toml_file: &Path, allow_overwrites: bool) {
    let current_dir = std::env::current_dir().expect("failed to get current directory");
    if let Err(e) = mbf_fastq_processor::run(toml_file, &current_dir, allow_overwrites) {
        eprintln!("Unfortunately, an error was detected and led to an early exit.\n");
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

fn validate_config_file(toml_path: PathBuf) {
    match mbf_fastq_processor::validate_config(&toml_path) {
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

#[allow(clippy::needless_pass_by_value)]
fn verify_config_file(toml_file: &Path, output_dir: Option<PathBuf>) {
    match mbf_fastq_processor::verify_outputs(toml_file, output_dir.as_deref()) {
        Ok(()) => {
            println!("✓ Verification passed: outputs match expected outputs");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Verification failed:\n");
            eprintln!(
                "# == Error Details ==\n{}",
                prettyify_error_message(&format!("{e:?}"))
            );
            std::process::exit(1);
        }
    }
}

fn run_interactive_mode(
    toml_path: PathBuf,
    head: Option<u64>,
    sample: Option<u64>,
    inspect: Option<u64>,
) {
    if let Err(e) =
        mbf_fastq_processor::interactive::run_interactive(&toml_path, head, sample, inspect)
    {
        eprintln!("Interactive mode error: {e:?}");
        std::process::exit(1);
    }
}

/// Find a single .toml file in the current directory that has both [input] and [output] sections
fn find_single_valid_toml() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    let mut valid_tomls = Vec::new();
    let mut any_tomls = false;

    for entry in ex::fs::read_dir(&current_dir)
        .with_context(|| format!("Failed to read directory: {}", current_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            // Try to read and parse the TOML to check for [input] and [output] sections
            any_tomls = true;
            if let Ok(content) = ex::fs::read_to_string(&path) {
                // Simple check: does it contain [input] and [output]?
                if content.contains("[input]") && content.contains("[output]") {
                    valid_tomls.push(path);
                }
            }
        }
    }

    match valid_tomls.len() {
        0 => {
            if any_tomls {
                bail!(
                    "TOML file(s) found in current directory, but none were valid TOML configuration files.\n A valid configuration must contain both [input] and [output] sections."
                );
            } else {
                bail!(
                    "No TOML file found in current directory by auto-detection.\n\
                     Add one to the current directory or specify a configuration file explicitly."
                );
            }
        }
        1 => {
            let path = valid_tomls
                .into_iter()
                .next()
                .expect("match arm guarantees vector has exactly one element");
            eprintln!("Auto-detected configuration file: {}", path.display());
            Ok(path)
        }
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
