use anyhow::{Context, Result, bail};
use clap::{Arg, ArgAction, Command, ValueHint, value_parser};
use clap_complete::{Generator, Shell, generate};
use human_panic::{Metadata, setup_panic};
use std::path::{Path, PathBuf};

/// Returned when the error has already been printed to stderr; callers should
/// exit with a non-zero code without printing anything further.
#[derive(Debug)]
pub struct EarlyExit;

impl std::fmt::Display for EarlyExit {
    //cov:excl-start
    //since not printing is the whole point
    #[mutants::skip]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
    //cov:excl-stop
}

impl std::error::Error for EarlyExit {}

fn early_exit() -> anyhow::Error {
    anyhow::Error::new(EarlyExit)
}

#[expect(clippy::too_many_lines, reason = "cli is complex")]
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

    Command::new("fastqrab")
        .version(version_string)
        .about(
            "Process FASTQ files with filtering, sampling, slicing, demultiplexing, and analysis",
        )
        .after_help(
            "Quick start:
    1. fastqrab cookbook # pick one
    2. fastqrab cookbook <no from 1> > pipeline.toml
    2. Edit pipeline.toml with your input files
    3. fastqrab process pipeline.toml

Docs:
    Visit https://tyberiusprime.github.io/fastqrab/ for in depth documentation
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
                        .help("Path to the TOML configuration file, or '-' to read from stdin")
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
                        .help("Path to the TOML configuration file to validate, or '-' to read from stdin")
                        .required(false)
                        .value_name("CONFIG_TOML")
                        .value_hint(ValueHint::FilePath),
                ),
        )

        .subcommand(
            Command::new("output-files")
                .about("List the output files a (valid) config would produce")
                .arg(
                    Arg::new("config")
                        .help("Path to the TOML configuration file to validate, or '-' to read from stdin")
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
                .arg(
                    Arg::new("unsafe-call-prep-sh")
                        .long("unsafe-call-prep-sh")
                        .help("Allow execution of prep.sh scripts (potentially unsafe, for internal testing.)")
                        .action(ArgAction::SetTrue),
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
                )
                .arg(
                    Arg::new("poll_interval")
                        .long("poll-interval")
                        .help("Polling interval in milliseconds (default: 1000)")
                        .value_name("MS")
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("max_runs")
                        .long("max-runs")
                        .help("Exit after processing N times (useful for testing)")
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
                    • Bash:       echo 'source <(fastqrab completions bash)' >> ~/.bashrc\n\
                    • Fish:       fastqrab completions fish > ~/.config/fish/completions/fastqrab.fish\n\
                    • Zsh:        echo 'source <(fastqrab completions zsh)' >> ~/.zshrc\n\
                    • PowerShell: fastqrab completions powershell | Out-String | Invoke-Expression"
                )
                .arg(
                    Arg::new("shell")
                        .help("Shell to generate completions for")
                        .required(true)
                        .value_parser(value_parser!(Shell))
                        .value_name("SHELL"),
                ),
        )
        .subcommand(
            Command::new("json-schema")
                .about("Generate a json schema for the configuration files")
                .long_about(
                    "Generate a json schema for your configuration files.\n\n\
                    Add #:schema <filepath.json> to your TOML\n\
                    to enable Tombi LSP based validation."
                )
        )
}

/// Generate shell completions and print to stdout
fn print_completions<G: Generator>(generator: G, cmd: &mut Command) {
    generate(
        generator,
        cmd,
        cmd.get_name().to_string(),
        &mut std::io::stdout(),
    );
}

fn print_schema() {
    let schema = fastqrab_steps::config::config_schema();
    println!(
        "{}",
        serde_json::to_string_pretty(&schema).expect("Schema could not be jsonified?")
    );
}

fn print_template(step: Option<&String>) {
    print!(
        "{}",
        crate::documentation::get_template(step.map(String::as_str))
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

fn print_cookbook(cookbook_number: Option<&String>) -> Result<()> {
    match cookbook_number {
        None => {
            // List all cookbooks
            println!("Available cookbooks:\n");
            let cookbooks = crate::cookbooks::list_cookbooks();
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
                .and_then(|num| crate::cookbooks::get_cookbook(num))
                .or_else(|| crate::cookbooks::get_cookbook_by_name(num_str));
            if let Some(cookbook) = cookbook {
                println!("{}", comment(cookbook.readme));
                println!("\n## Configuration (input.toml)\n");
                println!("{}", cookbook.toml);
            } else {
                bail!(
                    "Cookbook {num_str} not found. \
                     Use 'cookbook' without argument to list all available cookbooks."
                );
            }
        }
    }
    Ok(())
}

fn handle_toml_arg(config_file: Option<&String>) -> Result<PathBuf> {
    match config_file {
        Some(path) => Ok(PathBuf::from(path)),
        None => match find_single_valid_toml() {
            Ok(path) => Ok(path),
            Err(e) => {
                eprintln!("Error: {e}");
                eprintln!(
                    "\nPlease specify a configuration file explicitly: \
                     fastqrab verify <config.toml>"
                );
                Err(early_exit())
            }
        },
    }
}

#[mutants::skip]
fn handle_friendly_panic() {
    //this will trigger a mutant false positive, since we're only testing it in nix tests (needs
    //the release binary)
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
            .homepage("https://github.com/TyberiusPrime/fastqrab")
            .support("Open a github issue at https://github.com/TyberiusPrime/fastqrab/issues/new and attach the crash report.")
    );
    }

    assert!(
        !std::env::args().any(|x| x == "--test-friendly-panic"),
        "friendly panic test!"
    );
}

/// # Panics
/// on friendly panic test
pub fn entry_point() -> Result<()> {
    // Internal decompressor dispatch: `fastqrab __decompressor <args...>`.
    // Checked before build_cli() so the subcommand never needs to be defined there.
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("__decompressor")) {
        return crate::decompressor::run();
    }

    // Support environment-based completion generation (modern approach)
    // Usage: COMPLETE=bash fastqrab
    if let Ok(shell_str) = std::env::var("COMPLETE")
        && let Ok(shell) = shell_str.parse::<Shell>()
    {
        let mut cmd = build_cli();
        print_completions(shell, &mut cmd);
        return Ok(());
    }

    handle_friendly_panic();
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
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"))?;
            let allow_overwrites = sub_matches.get_flag("allow-overwrite");
            process_from_toml_file(&toml_path, allow_overwrites)?;
        }
        Some(("template", sub_matches)) => {
            let section = sub_matches.get_one::<String>("section");
            print_template(section);
        }
        Some(("cookbook", sub_matches)) => {
            let number = sub_matches.get_one::<String>("number");
            print_cookbook(number)?;
        }
        Some(("list-steps", _)) => {
            print!("{}", crate::list_steps::format_steps_list());
        }
        Some(("version", _)) => {
            print_version();
        }
        Some(("validate", sub_matches)) => {
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"))?;
            validate_config_file(&toml_path)?;
        }
        Some(("output-files", sub_matches)) => {
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"))?;
            output_files(&toml_path)?;
        }
        Some(("verify", sub_matches)) => {
            let output_dir = sub_matches.get_one::<String>("output-dir");
            let unsafe_prep = sub_matches.get_flag("unsafe-call-prep-sh");

            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"))?;
            verify_config_file(&toml_path, output_dir.map(PathBuf::from), unsafe_prep)?;
        }
        Some(("interactive", sub_matches)) => {
            if sub_matches
                .get_one::<String>("config")
                .is_some_and(|s| s == "-")
            {
                bail!("interactive mode cannot read configuration from stdin.");
            }
            let toml_path = handle_toml_arg(sub_matches.get_one::<String>("config"))?;
            let head = sub_matches.get_one::<u64>("head").copied();
            let sample = sub_matches.get_one::<u64>("sample").copied();
            let inspect = sub_matches.get_one::<u64>("inspect").copied();
            let poll_interval = sub_matches.get_one::<u64>("poll_interval").copied();
            let max_runs = sub_matches.get_one::<u64>("max_runs").copied();
            run_interactive_mode(&toml_path, head, sample, inspect, poll_interval, max_runs)?;
        }
        Some(("completions", sub_matches)) => {
            if let Some(shell) = sub_matches.get_one::<Shell>("shell") {
                let mut cmd = build_cli();
                print_completions(*shell, &mut cmd);
            }
        }
        Some(("json-schema", _sub_matches)) => {
            print_schema();
        }
        _ => {
            // This shouldn't happen due to arg_required_else_help, but just in case
            build_cli().print_help()?;
            bail!("no subcommand provided");
        }
    }
    Ok(())
}

fn print_version() {
    println!(
        "{} (git: {})",
        env!("CARGO_PKG_VERSION"),
        option_env!("COMMIT_HASH").unwrap_or("unknown")
    );
}

fn process_from_toml_file(toml_file: &Path, allow_overwrites: bool) -> Result<()> {
    let current_dir = std::env::current_dir().context("failed to get current directory")?;
    if let Err(e) = crate::run(toml_file, &current_dir, allow_overwrites) {
        eprintln!("Unfortunately, an error was detected and led to an early exit.\n");
        eprintln!("# == Error Details ==\n{e:?}");
        return Err(early_exit());
    }
    Ok(())
}

fn validate_config_file(toml_path: &Path) -> Result<()> {
    match crate::validate_config(toml_path, toml_path == Path::new("-")) {
        Ok(warnings) => {
            if warnings.is_empty() {
                println!("✓ Configuration is valid");
            } else {
                println!("✓ Configuration is valid (with warnings)");
                for warning in warnings {
                    eprintln!("Warning: {warning}");
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Configuration validation failed:\n");
            eprintln!("# == Error Details ==\n{e:?}");
            Err(early_exit())
        }
    }
}

fn output_files(toml_path: &Path) -> Result<()> {
    match crate::list_config_output_files(toml_path) {
        Ok(listing) => {
            if listing.files.is_empty() {
                println!("No output files would be produced by this configuration.");
            } else {
                println!("This configuration would produce the following output files:");
                for file in &listing.files {
                    println!("  {file}");
                }
                if listing.any_chunked {
                    eprintln!(
                        "\nNote: chunked outputs are listed by their first chunk ('.0' suffix). \
                         The run may emit further numbered chunks ('.1', '.2', ...) depending on \
                         how many reads are written. If more digits are needed .1 is renamed .01 so \
                        the number of digits remains constant."
                    );
                }
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("Configuration validation failed:\n");
            eprintln!("# == Error Details ==\n{e:?}");
            Err(early_exit())
        }
    }
}

#[expect(clippy::needless_pass_by_value, reason = "it's only a test")]
fn verify_config_file(
    toml_file: &Path,
    output_dir: Option<PathBuf>,
    unsafe_prep: bool,
) -> Result<()> {
    match crate::verify_outputs(toml_file, output_dir.as_deref(), unsafe_prep) {
        Ok(()) => {
            println!("✓ Verification passed: outputs match expected outputs");
            Ok(())
        }
        Err(e) => {
            eprintln!("Verification failed:\n");
            eprintln!("# == Error Details ==\n{e:?}");
            Err(early_exit())
        }
    }
}

fn run_interactive_mode(
    toml_path: &Path,
    head: Option<u64>,
    sample: Option<u64>,
    inspect: Option<u64>,
    poll_interval: Option<u64>,
    max_runs: Option<u64>,
) -> Result<()> {
    if let Err(e) = crate::interactive::run_interactive(
        toml_path,
        head,
        sample,
        inspect,
        poll_interval,
        max_runs,
    ) {
        eprintln!("Interactive mode error: {e:?}");
        return Err(early_exit());
    }
    Ok(())
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
            } // cov:excl-line
        }
    }

    match valid_tomls.len() {
        0 => {
            if any_tomls {
                bail!(
                    "TOML file(s) found in current directory, \
                    but none were valid TOML configuration files.\n \
                    A valid configuration must contain both [input] and [output] sections.\n \
                    Symlinks are not being followed."
                );
            }
            bail!(
                "No TOML file found in current directory by auto-detection.\n\
                 Add one to the current directory or specify a configuration file explicitly."
            );
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
