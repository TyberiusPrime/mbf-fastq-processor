mod anchor;
mod gc_content;
mod iupac;
mod iupac_suffix;
mod length;
mod low_complexity;
mod low_quality_end;
mod low_quality_start;
mod mean_quality;
mod n_count;
mod poly_tail;
mod qualified_bases;
mod regex;
mod region;
mod regions;
mod regions_of_low_quality;

pub mod tag;

pub use anchor::Anchor;
pub use gc_content::GCContent;
pub use iupac::IUPAC;
pub use iupac_suffix::IUPACSuffix;
pub use length::Length;
pub use low_complexity::LowComplexity;
pub use low_quality_end::LowQualityEnd;
pub use low_quality_start::LowQualityStart;
pub use mean_quality::MeanQuality;
pub use n_count::NCount;
pub use poly_tail::PolyTail;
pub use qualified_bases::QualifiedBases;
pub use regex::Regex;
pub use region::Region;
pub use regions::Regions;
pub use regions_of_low_quality::RegionsOfLowQuality;

use crate::{
    config::{Segment, SegmentIndex, SegmentIndexOrAll, SegmentOrAll},
    dna::TagValue,
    io,
};
use std::collections::HashMap;

pub(crate) fn extract_tags(
    block: &mut io::FastQBlocksCombined,
    segment: &SegmentIndex,
    label: &str,
    f: impl Fn(&mut io::WrappedFastQRead) -> Option<crate::dna::Hits>,
) {
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }
    let mut out = Vec::new();

    let f2 = |read: &mut io::WrappedFastQRead| {
        out.push(match f(read) {
            Some(hits) => TagValue::Sequence(hits),
            None => TagValue::Missing,
        });
    };
    block.segments[segment.get_index()].apply(f2);

    block.tags.as_mut().unwrap().insert(label.to_string(), out);
}

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

pub(crate) fn extract_bool_tags<F>(
    block: &mut io::FastQBlocksCombined,
    segment: &SegmentIndex,
    label: &str,
    mut extractor: F,
) where
    F: FnMut(&io::WrappedFastQRead) -> bool,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    let mut values = Vec::new();
    let f = |read: &mut io::WrappedFastQRead| {
        values.push(TagValue::Bool(extractor(read)));
    };
    block.segments[segment.get_index()].apply(f);

    block
        .tags
        .as_mut()
        .unwrap()
        .insert(label.to_string(), values);
}

pub(crate) fn extract_numeric_tags_plus_all<F>(
    segment: &SegmentIndexOrAll,
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

pub(crate) fn extract_bool_tags_plus_all<F, G>(
    block: &mut io::FastQBlocksCombined,
    segment: &SegmentIndexOrAll,
    label: &str,
    extractor_single: F,
    mut extractor_all: G,
) where
    F: FnMut(&io::WrappedFastQRead) -> bool,
    G: FnMut(&Vec<io::WrappedFastQRead>) -> bool,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    let target: Result<SegmentIndex, _> = segment.try_into();
    if let Ok(target) = target {
        // Handle single target case
        extract_bool_tags(block, &target, label, extractor_single);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(&molecule.segments);
            values.push(TagValue::Bool(value));
        }
        block
            .tags
            .as_mut()
            .unwrap()
            .insert(label.to_string(), values);
    }
}
