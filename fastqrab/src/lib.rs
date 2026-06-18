mod bam_merge;
pub mod cli;
pub mod cookbooks;
pub mod documentation;
pub mod interactive;
pub mod io;
pub mod list_steps;
mod output;
mod pipeline;
mod pipeline_workpool;

pub use fastqrab_steps::config;
pub use fastqrab_steps::demultiplex;
pub(crate) use fastqrab_steps::transformations;

pub use cli::output_files::list_config_output_files;
pub use cli::process::run;
pub use cli::validate::validate_config;
pub use cli::verify::decompress_file; // used by parser tests
pub use cli::verify::verify_outputs;
