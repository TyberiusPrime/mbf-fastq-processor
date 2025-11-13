//! Transformation steps for mbf-fastq-processor pipelines
//!
//! This crate contains all the transformation logic:
//! - Filtering (quality, length, content, duplicates)
//! - Trimming and editing
//! - Demultiplexing
//! - Tagging and extraction
//! - Validation
//! - Reporting
//! - Calculations

#![allow(clippy::redundant_else)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::single_match_else)]
#![allow(non_camel_case_types)] // to make eserde and _Internal* shut up

pub mod calc;
pub mod convert;
pub mod demultiplex;
pub mod edits;
pub mod extract;
pub mod filters;
pub mod hamming_correct;
pub mod prelude;
pub mod reports;
pub mod tag;
pub mod validation;

use anyhow::{Result, bail};
use enum_dispatch::enum_dispatch;
use eserde::Deserialize;
use schemars::JsonSchema;
use serde_valid::Validate;

pub use mbf_core::{FastQElement, FastQRead, Position, WrappedFastQRead, WrappedFastQReadMut};
pub use mbf_config::{Config, Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};

#[derive(Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegionDefinition {
    #[serde(default)]
    pub segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndex>,

    pub start: usize,
    #[validate(minimum = 1)]
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum TagValueType {
    Location, // string + in-sequence-location
    String,   // just a piece of text
    Numeric,
    Bool,
}

impl TagValueType {
    pub fn compatible(self, other: TagValueType) -> bool {
        self == other
    }
}

impl std::fmt::Display for TagValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TagValueType::Location => write!(f, "Location"),
            TagValueType::String => write!(f, "String"),
            TagValueType::Numeric => write!(f, "Numeric"),
            TagValueType::Bool => write!(f, "Boolean"),
        }
    }
}

// Main transformation trait and enum would go here
// This is just a skeleton showing the structure
