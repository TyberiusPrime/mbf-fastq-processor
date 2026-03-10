pub mod cli;
pub mod cookbooks;
pub mod documentation;
pub mod interactive;
pub mod io;
pub mod list_steps;
mod output;
mod pipeline;
mod pipeline_workpool;

pub use mbf_fastq_processor_steps::config;
pub use mbf_fastq_processor_steps::demultiplex;
pub(crate) use mbf_fastq_processor_steps::transformations;

pub use cli::process::run;
pub use cli::validate::validate_config;
pub use cli::verify::decompress_file; // used by parser tests
pub use cli::verify::verify_outputs;
