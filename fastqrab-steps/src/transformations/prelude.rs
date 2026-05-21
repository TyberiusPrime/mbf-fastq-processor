pub(crate) use super::{
    FinalizeReportResult, FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter,
    PartialTransformation, Step, TagUsageInfo, TagUser, edits::get_bool_vec_from_tag,
    reproducible_cuckoofilter,
};
pub(crate) use crate::config::{
    DenyName, PartialConfig, TagMetadata, ValidateSegment, ValidateTagLabel,
};

pub use fastqrab_config::{
    ConditionalTagLabel, FileFormat, RemovedTags, TagLabel, TagValueType, ToDeclaredTag, ToUsedTag,
    ToUsedTags, UsedTag,
    dna::{Hit, HitRegion, Hits, TagColumn},
    offer_alternatives,
    segments::{
        ResolvedSourceAll,
        ResolvedSourceNoAll,
        SegmentIndex,
        SegmentIndexOrAll,
        //SegmentOrNameIndex,
    },
};

pub(crate) use crate::demultiplex::{
    DemultiplexBarcodes, DemultiplexedData, OptDemultiplex, StepOutputFiles,
};

pub(crate) use anyhow::{Context, Result, bail};
pub(crate) use fastqrab_io::io::{
    FastQBlocksCombined,
    output::chunked_writer::{
        ChunkPolicy, ChunkedRecordWriter, OutputDeclaration, SinkConfig, WriteTargetConfig,
    },
    reads::NewLocation,
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

pub type FxIndexMap<K, V> = IndexMap<K, V, rustc_hash::FxBuildHasher>;
