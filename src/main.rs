use human_panic::{setup_panic, Metadata};
use std::path::PathBuf;

use anyhow::{Context, Result};

fn print_usage(exit_code: i32) -> ! {
    eprintln!(
        "Usage: 
    {} <config.toml> [working_directory] # normal operation
    {} --version # output version and exit(0)

",
        std::env::args().next().unwrap(),
        std::env::args().next().unwrap()
    );
    std::process::exit(exit_code);
}

fn main() -> Result<()> {
    setup_panic!(
        Metadata::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
            //.authors("My Company Support <support@mycompany.com>")
            .homepage("https://github.com/TyberiusPrime/mbf_fastq_processor")
            .support("Open a github issue at https://github.com/TyberiusPrime/mbf_fastq_processor/issues/new and attach the crash report.")
    );

    if std::env::args().any(|x| x == "--test-friendly-panic") {
        panic!("friendly panic test!");
    }

    if std::env::args().any(|x| x == "--version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
    if std::env::args().any(|x| x == "--help") {
        print_usage(0);
    }
    if std::env::args().len() < 2 {
        print_usage(1);
    }
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let toml_file = PathBuf::from(toml_file);
    let current_dir = std::env::args()
        .nth(2)
        .map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);
    if let Err(e) = mbf_fastq_processor::run(&toml_file, &current_dir) {
        eprintln!(
            "Unfortunatly an error was detected and lead to an early exit.\n\nDetails: {:?}",
            e
        );
        std::process::exit(1);
    }
    Ok(())
}
