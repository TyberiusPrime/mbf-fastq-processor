pub(crate) use super::{
    FinalizeReportResult, FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter,
    PartialTransformation, Step, TagUsageInfo, TagUser, edits::get_bool_vec_from_tag,
    reproducible_cuckoofilter,
};
pub(crate) use crate::config::{PartialConfig, TagMetadata, ValidateSegment};

pub use fastqrab_config::{
    ConditionalTagLabel, RemovedTags, TagLabel, TagValueType, ToDeclaredTag, ToUsedTag, ToUsedTags,
    UsedTag,
    dna::{Hit, HitRegion, Hits, TagValue},
    offer_alternatives,
    segments::{
        ResolvedSourceAll, ResolvedSourceNoAll, SegmentIndex, SegmentIndexOrAll, SegmentOrNameIndex,
    },
};

pub(crate) use crate::demultiplex::{
    DemultiplexBarcodes, DemultiplexedData, DemultiplexedOutputFiles, OptDemultiplex,
};

pub(crate) use anyhow::{Context, Result, bail};
pub(crate) use fastqrab_io::io::{
    FastQBlocksCombined, output::compressed_output::OutputWriter, reads::NewLocation,
    reads::WrappedFastQReadCommon,
};

pub(crate) use bstr::{BStr, BString};
pub(crate) use schemars::JsonSchema;

pub(crate) type DemultiplexTag = crate::demultiplex::Tag;

pub(crate) use std::sync::{Arc, Mutex};

pub(crate) use std::path::Path;

pub(crate) const ANY_TAG_TYPE: &[TagValueType] = &[
    TagValueType::String,
    TagValueType::Bool,
    TagValueType::Numeric((None, None)),
    TagValueType::Location,
];

pub use indexmap::IndexMap;
pub use toml_pretty_deser::prelude::*;
pub use typed_floats::tf64::NonNaN;
