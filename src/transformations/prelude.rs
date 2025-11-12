pub use super::{FinalizeReportResult, InputInfo, Step, TagValueType, Transformation};
pub use crate::config::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};
pub use crate::demultiplex::{DemultiplexBarcodes, DemultiplexedData, OptDemultiplex};

pub use crate::io::FastQBlocksCombined;
pub use anyhow::{Context, Result, anyhow, bail};

pub use bstr::BString;
pub use schemars::JsonSchema;

pub type DemultiplexTag = crate::demultiplex::Tag;
