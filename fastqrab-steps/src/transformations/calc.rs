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

pub use base_content::{BaseContent, PartialBaseContent};
pub use complexity::{Complexity, PartialComplexity};
pub use convert_to_rate::{ConvertToRate, PartialConvertToRate};
pub use expected_error::{ExpectedError, PartialExpectedError};
use fastqrab_config::{
    TagLabel,
    dna::TagValue,
    segments::{SegmentIndex, SegmentIndexOrAll},
};
pub use gc_content::{GCContent, PartialGCContent};
pub use kmers::{Kmers, PartialKmers};
pub use length::{Length, PartialLength};
pub use n_content::{NContent, PartialNContent};
pub use qualified_bases::{PartialQualifiedBases, QualifiedBases};

pub(crate) fn extract_numeric_tags<F>(
    segment: SegmentIndex,
    label: &TagLabel,
    mut extractor: F,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&WrappedFastQRead) -> f64,
{
    let mut values = Vec::with_capacity(block.segments[segment.get_index()].len()); //7% speed up
    let f = |read: &mut WrappedFastQRead| {
        values.push(TagValue::Numeric(extractor(read)));
    };

    block.segments[segment.get_index()].apply(f);
    block.tags.insert(label.clone(), values);
}

pub(crate) fn extract_numeric_tags_plus_all<F>(
    segment: SegmentIndexOrAll,
    label: &TagLabel,
    extractor_single: F,
    mut extractor_all: impl FnMut(&Vec<WrappedFastQRead>) -> f64,
    block: &mut FastQBlocksCombined,
) where
    F: FnMut(&WrappedFastQRead) -> f64,
{
    if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
        // Handle single target case
        extract_numeric_tags(target, label, extractor_single, block);
    } else {
        // Handle "All" target case
        let mut values = Vec::with_capacity(block.segments[0].len());
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(&molecule.segments);
            values.push(TagValue::Numeric(value));
        }
        block.tags.insert(label.clone(), values);
    }
}
