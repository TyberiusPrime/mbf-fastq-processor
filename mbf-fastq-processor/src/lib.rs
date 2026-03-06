pub mod cli;
pub mod config;
pub mod cookbooks;
pub mod demultiplex;
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
