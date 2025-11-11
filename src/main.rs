use allocation_counter::measure;
use human_panic::{setup_panic, Metadata};
use regex::Regex;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use anyhow::{Context, Result};

#[repr(u8)]
#[derive(Clone, Copy)]
enum StdoutOrStderr {
    Stdout,
    Stderr,
}

fn print_usage(exit_code: i32, stdout_or_stderr: StdoutOrStderr) -> ! {
    let this_cmd = std::env::args().next().unwrap();
    let this_cmd = PathBuf::from(this_cmd)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("mbf-fastq-processor")
        .to_string();
    let usg = format!(
        "Usage:
    process FASTQ files:
        {this_cmd} process <config.toml> [--allow-overwrite]

    output configuration template / subsection:
        {this_cmd} template [section]

    list available cookbooks:
        {this_cmd} cookbook

    show specific cookbook:
        {this_cmd} cookbook <number>

    list available transformation steps:
        {this_cmd} list-steps

    output version:
        {this_cmd} version # output version and exit(0)


Minimimal configuration.toml:
```toml
    [input]
        read_1 = 'input_R1.fq.gz'

    [[step]]
        action = 'Report'
        label = 'my_report'
        count = true

    [output]
        prefix = 'output'
        report_html = true
```
"
    );
    match stdout_or_stderr {
        StdoutOrStderr::Stdout => print!("{usg}"),
        StdoutOrStderr::Stderr => eprint!("{usg}"),
    }
    std::process::exit(exit_code);
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
        Metadata::new(env!("CARGO_PKG_NAME"), format!("{} (git: {})", 
            env!("CARGO_PKG_VERSION"),
            env!("COMMIT_HASH")
        ))
            //.authors("My Company Support <support@mycompany.com>")
            .homepage("https://github.com/TyberiusPrime/mbf-fastq-processor")
            .support("Open a github issue at https://github.com/TyberiusPrime/mbf-fastq-processor/issues/new and attach the crash report.")
    );
    }

    assert!(
        !std::env::args().any(|x| x == "--test-friendly-panic"),
        "friendly panic test!"
    );

    if std::env::args().any(|x| x == "--version") {
        print_version_and_exit();
    }

    if std::env::args().any(|x| x == "--help") {
        print_usage(1, StdoutOrStderr::Stdout);
    }

    if std::env::args().len() < 2 {
        print_usage(1, StdoutOrStderr::Stderr);
    }

    let command = std::env::args().nth(1).unwrap();

    match command.as_str() {
        "template" => {
            let step = std::env::args().nth(2);
            print_template(step.as_ref());
            std::process::exit(0);
        }
        "version" => {
            print_version_and_exit();
        }
        "cookbook" => {
            let cookbook_number = std::env::args().nth(2);
            print_cookbook(cookbook_number.as_ref());
            std::process::exit(0);
        }
        "list-steps" => {
            print!("{}", mbf_fastq_processor::list_steps::format_steps_list());
            std::process::exit(0);
        }
        "process" => {
            if std::env::args().len() < 3 {
                eprintln!("Error: 'process' command requires a config file path");
                print_usage(1, StdoutOrStderr::Stderr);
            }
            let toml_file = std::env::args()
                .nth(2)
                .context("Second argument must be a toml file path.")?;
            let allow_overwrites = std::env::args().any(|x| x == "--allow-overwrite");
            run_with_optional_measure(|| process_from_toml_file(&toml_file, allow_overwrites));
        }
        _ => {
            // For backward compatibility, try to parse as old format (direct config file)
            if command.ends_with(".toml") {
                run_with_optional_measure(|| process_from_toml_file(&command, false));
            } else {
                eprintln!("Invalid command");
                print_usage(1, StdoutOrStderr::Stderr);
            }
        }
    }
    Ok(())
}

fn print_version_and_exit() {
    let version = env!("CARGO_PKG_VERSION");
    let commit_hash = env!("COMMIT_HASH");

    // Show short commit (first 8 characters)
    let short_commit = if commit_hash.len() >= 8 {
        &commit_hash[0..8]
    } else {
        commit_hash
    };
    println!("{} ({})", version, short_commit);
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
