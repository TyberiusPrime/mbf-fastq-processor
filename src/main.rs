use std::path::PathBuf;

use anyhow::{Context, Result};

fn main() -> Result<()> {
    let toml_file = std::env::args()
        .nth(1)
        .context("First argument must be a toml file path.")?;
    let toml_file = PathBuf::from(toml_file);
    let current_dir = PathBuf::from(std::env::current_dir()?);
    mbf_fastq_processor::run(&toml_file, &current_dir)
}
