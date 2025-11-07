mod base_content;
mod complexity;
mod expected_error;
mod kmers;
mod length;
mod n_count;
mod qualified_bases;

mod gc_content;
use std::collections::HashMap;

use crate::{
    config::{SegmentIndex, SegmentIndexOrAll},
    dna::TagValue,
    io,
};

pub use base_content::BaseContent;
pub use complexity::Complexity;
pub use expected_error::ExpectedError;
pub use gc_content::GCContent;
pub use kmers::Kmers;
pub use length::Length;
pub use n_count::NCount;
pub use qualified_bases::QualifiedBases;

pub(crate) fn extract_numeric_tags<F>(
    segment: SegmentIndex,
    label: &str,
    mut extractor: F,
    block: &mut io::FastQBlocksCombined,
) where
    F: FnMut(&io::WrappedFastQRead) -> f64,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    let mut values = Vec::new();
    let f = |read: &mut io::WrappedFastQRead| {
        values.push(TagValue::Numeric(extractor(read)));
    };

    block.segments[segment.get_index()].apply(f);
    block
        .tags
        .as_mut()
        .unwrap()
        .insert(label.to_string(), values);
}

pub(crate) fn extract_numeric_tags_plus_all<F>(
    segment: SegmentIndexOrAll,
    label: &str,
    extractor_single: F,
    mut extractor_all: impl FnMut(&Vec<io::WrappedFastQRead>) -> f64,
    block: &mut io::FastQBlocksCombined,
) where
    F: FnMut(&io::WrappedFastQRead) -> f64,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    if let Ok(target) = segment.try_into() as Result<SegmentIndex, _> {
        // Handle single target case
        extract_numeric_tags(target, label, extractor_single, block);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(&molecule.segments);
            values.push(TagValue::Numeric(value));
        }
        block
            .tags
            .as_mut()
            .unwrap()
            .insert(label.to_string(), values);
    }
}
