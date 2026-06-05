use bstr::BStr;
use fastqrab_dna::dna::TagColumn;
use fastqrab_io::io::{FastQBlocksCombined, WrappedFastQRead};

mod base_content;
mod complexity;
mod convert_to_rate;
mod expected_error;
mod kmers;
mod length;
mod n_content;
mod qualified_bases;

mod gc_content;
mod worst_quality;

pub use base_content::{BaseContent, PartialBaseContent};
pub use complexity::{Complexity, PartialComplexity};
pub use convert_to_rate::{ConvertToRate, PartialConvertToRate};
pub use expected_error::{ExpectedError, PartialExpectedError};
use fastqrab_config::{
    TagLabel,
    segments::{SegmentIndex, SegmentIndexOrAll},
};
pub use gc_content::{GCContent, PartialGCContent};
pub use kmers::{Kmers, PartialKmers};
pub use length::{Length, PartialLength};
pub use n_content::{NContent, PartialNContent};
pub use qualified_bases::{PartialQualifiedBases, QualifiedBases};
pub use worst_quality::{PartialWorstQuality, WorstQuality};

pub(crate) fn extract_numeric_tags_from_sequences<F>(
    segment: SegmentIndex,
    label: &TagLabel,
    mut extractor: F,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&BStr) -> f64,
{
    let mut values: Vec<f64> = Vec::with_capacity(block.segments[segment.as_index()].len());
    for seq in block.segments[segment.as_index()].seq_quals.iter_seq() {
        values.push(extractor(seq));
    }
    block.tags.insert(label.clone(), TagColumn::Numeric(values));
}

pub(crate) fn extract_numeric_tags_from_qualities<F>(
    segment: SegmentIndex,
    label: &TagLabel,
    mut extractor: F,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&BStr) -> f64,
{
    let mut values: Vec<f64> = Vec::with_capacity(block.segments[segment.as_index()].len());
    for seq in block.segments[segment.as_index()].seq_quals.iter_qual() {
        values.push(extractor(seq));
    }
    block.tags.insert(label.clone(), TagColumn::Numeric(values));
}

pub(crate) fn extract_numeric_tags_plus_all_from_sequences<F>(
    segment: SegmentIndexOrAll,
    label: &TagLabel,
    extractor_single: F,
    mut extractor_all: impl FnMut(&Vec<&BStr>) -> f64,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&BStr) -> f64,
{
    if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
        // Handle single target case
        extract_numeric_tags_from_sequences(target, label, extractor_single, block);
    } else {
        // Handle "All" target case
        let mut values = Vec::with_capacity(block.segments[0].len());
        let iters = block.segments.iter().map(|chunk| chunk.seq_quals.iter_seq());
        for row in iters {
            let argument: Vec<&BStr> = row.collect();
            let value = extractor_all(&argument);
            values.push(value);
        }
        block.tags.insert(label.clone(), TagColumn::Numeric(values));
    }
}

pub(crate) fn extract_numeric_tags_plus_all_from_qualities<F>(
    segment: SegmentIndexOrAll,
    label: &TagLabel,
    extractor_single: F,
    mut extractor_all: impl FnMut(&Vec<&BStr>) -> f64,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&BStr) -> f64,
{
    if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
        // Handle single target case
        extract_numeric_tags_from_qualities(target, label, extractor_single, block);

    } else {
        // Handle "All" target case
        let mut values = Vec::with_capacity(block.segments[0].len());
        let iters = block.segments.iter().map(|chunk| chunk.seq_quals.iter_qual());
        for row in iters {
            let argument: Vec<&BStr> = row.collect();
            let value = extractor_all(&argument);
            values.push(value);
        }
        block.tags.insert(label.clone(), TagColumn::Numeric(values));
    }
}
