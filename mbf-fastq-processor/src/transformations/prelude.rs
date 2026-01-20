pub use super::{
    ConditionalTag, FinalizeReportResult, InputInfo, ResolvedSourceAll, ResolvedSourceNoAll, Step, TagValueType,
    Transformation, get_bool_vec_from_tag,
};
pub use crate::config::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll, TagMetadata};
pub use crate::demultiplex::{
    DemultiplexBarcodes, DemultiplexedData, DemultiplexedOutputFiles, OptDemultiplex, OutputWriter,
};

pub use crate::io::{FastQBlocksCombined, reads::NewLocation};
pub use anyhow::{Context, Result, anyhow, bail};

pub use bstr::BString;
pub use schemars::JsonSchema;

pub type DemultiplexTag = crate::demultiplex::Tag;
pub use std::collections::BTreeMap;

pub use std::sync::{Arc, Mutex};

pub use std::path::Path;

pub const ANY_TAG_TYPE: &[TagValueType] = &[
    TagValueType::String,
    TagValueType::Bool,
    TagValueType::Numeric,
    TagValueType::Location,
];
