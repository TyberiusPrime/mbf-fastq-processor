#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]
#![allow(clippy::default_trait_access)]

pub mod cli;
pub mod config;
pub mod cookbooks;
pub mod demultiplex;
mod dna;
pub mod documentation;
pub mod interactive;
pub mod io;
pub mod list_steps;
mod output;
mod pipeline;
mod pipeline_workpool;
mod transformations;

pub use cli::process::run;
pub use cli::validate::validate_config;
pub use cli::verify::decompress_file; // used by parser tests
pub use cli::verify::verify_outputs; 

#[must_use]
#[mutants::skip]
fn get_number_of_cores() -> usize {
    std::env::var("MBF_FASTQ_PROCESSOR_NUM_CPUS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| num_cpus::get())
}

fn join_nonempty<'a>(parts: impl IntoIterator<Item = &'a str>, separator: &str) -> String {
    let mut iter = parts.into_iter().filter(|part| !part.is_empty());
    let mut result = String::new();
    if let Some(first) = iter.next() {
        result.push_str(first);
        for part in iter {
            result.push_str(separator);
            result.push_str(part);
        }
    }
    result
}
