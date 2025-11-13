//! TOML configuration parsing and validation for mbf-fastq-processor
//!
//! This crate handles:
//! - Parsing TOML configuration files
//! - Validating configuration structure
//! - Input/output configuration
//! - Segment definitions
//! - Options and defaults

#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]

pub mod deser;
pub mod input;
pub mod options;
pub mod output;
pub mod segments;

use anyhow::Result;
use serde_valid::Validate;
use eserde::Deserialize;
use schemars::JsonSchema;

pub use segments::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};
pub use input::{InputConfig, StructuredInput, InputOptions};
pub use output::OutputConfig;
pub use options::Options;

/// Magic path constant for stdin
pub const STDIN_MAGIC_PATH: &str = "-";

#[derive(Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub input: InputConfig,
    #[serde(default)]
    pub options: Options,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub step: Vec<serde_json::Value>,  // Will be deserialized by transformations crate
    pub output: Option<OutputConfig>,
}

impl Config {
    pub fn check(&mut self) -> Result<()> {
        self.validate()?;
        self.input.initialize_structured_input(&self.options)?;
        Ok(())
    }

    pub fn check_for_validation(&mut self) -> Result<()> {
        self.validate()?;
        self.input.initialize_structured_input(&self.options)?;
        Ok(())
    }
}
