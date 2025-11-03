use allocation_counter::measure;
use human_panic::{Metadata, setup_panic};
use std::{collections::HashSet, path::PathBuf};

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
    {this_cmd} process <config.toml> [working_directory] [--allow-overwrite]# process FastQ files
    {this_cmd} template # output configuration template
    {this_cmd} version # output version and exit(0)
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

#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn main() -> Result<()> {
    if std::env::var("NO_FRIENDLY_PANIC").is_err() && std::env::var("RUST_BACKTRACE").is_err() {
        setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
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
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
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
            println!("{}", env!("CARGO_PKG_VERSION"));
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

fn process_from_toml_file(toml_file: &str, allow_overwrites: bool) {
    let toml_file = PathBuf::from(toml_file);
    let current_dir = std::env::args()
        .nth(3)
        .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);
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

        eprintln!("# == Error Details ==\n{e:?}",);
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
