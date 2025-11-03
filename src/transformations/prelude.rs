pub use super::{FinalizeReportResult, InputInfo, Step, TagValueType, Transformation};
pub use crate::config::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};
pub use crate::demultiplex::{DemultiplexBarcodes, OptDemultiplex};

pub use crate::io::FastQBlocksCombined;
pub use anyhow::{Context, Result, anyhow, bail};

pub use bstr::BString;

pub type DemultiplexTag = crate::demultiplex::Tag;
