use std::{borrow::Cow, cell::RefCell};

use bstr::{BStr, BString};
use fastqrab_dna::dna::TagColumn;
use indexmap::IndexMap;

use super::prelude::DemultiplexTag;
use fastqrab_config::{
    TagLabel,
    dna::{HitDraft, LocationColumn},
    segments::{SegmentIndex, SegmentIndexOrAll},
};
use fastqrab_io::io::{FastQBlocksCombined, WrappedFastQRead};

mod iupac;
mod iupac_suffix;
mod iupac_with_indel;
mod longest_poly_x;
mod low_quality_end;
mod low_quality_start;
mod poly_tail;
mod regex;
mod region;
mod regions;
mod regions_of_low_quality;
pub mod tag;

pub use iupac::{IUPAC, PartialIUPAC};
pub use iupac_suffix::{IUPACSuffix, PartialIUPACSuffix};
pub use iupac_with_indel::{IUPACWithIndel, PartialIUPACWithIndel};
pub use longest_poly_x::{LongestPolyX, PartialLongestPolyX};
pub use low_quality_end::{LowQualityEnd, PartialLowQualityEnd};
pub use low_quality_start::{LowQualityStart, PartialLowQualityStart};
pub use poly_tail::{PartialPolyTail, PolyTail};
pub use regex::{PartialRegex, Regex};
pub use region::{PartialRegion, Region};
pub use regions::{PartialRegions, Regions};
pub use regions_of_low_quality::{PartialRegionsOfLowQuality, RegionsOfLowQuality};

pub(crate) fn extract_region_tags(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&mut WrappedFastQRead) -> Option<HitDraft>,
) {
    let mut col = LocationColumn::new();

    let f2 = |read: &mut WrappedFastQRead| match f(read) {
        Some(draft) => col.push_single(draft.location, &draft.sequence),
        None => col.push_none(),
    };
    block.segments[segment.as_index()].apply(f2);

    block.tags.insert(label.clone(), TagColumn::Location(col));
}

// pub(crate) fn extract_string_tags(
//     block: &mut FastQBlocksCombined,
//     segment: SegmentIndex,
//     label: &TagLabel,
//     f: impl Fn(&mut WrappedFastQRead) -> Option<BString>,
// ) {
//     let mut out = Vec::new();
//
//     let f2 = |read: &mut WrappedFastQRead| {
//         out.push(match f(read) {
//             Some(hits) => TagValue::String(hits),
//             None => TagValue::Missing,
//         });
//     };
//     block.segments[segment.as_index()].apply(f2);
//
//     block.tags.insert(label.clone(), out);
// }

pub(crate) fn extract_region_tags_using_tags(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&mut WrappedFastQRead, usize, &IndexMap<TagLabel, TagColumn>) -> Option<HitDraft>,
) {
    let mut col = LocationColumn::new();

    let mut read_no = RefCell::new(0usize);
    let f2 = |read: &mut WrappedFastQRead| {
        match f(read, *read_no.borrow(), &mut block.tags) {
            Some(draft) => col.push_single(draft.location, &draft.sequence),
            None => col.push_none(),
        }
        *read_no.get_mut() += 1;
    };
    block.segments[segment.as_index()].apply(f2);

    block.tags.insert(label.clone(), TagColumn::Location(col));
}

pub(crate) fn extract_string_tags_using_tags(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&mut WrappedFastQRead, usize, &IndexMap<TagLabel, TagColumn>) -> Option<BString>,
) {
    let mut out = Vec::new();
    let mut read_no = RefCell::new(0usize);

    let f2 = |read: &mut WrappedFastQRead| {
        out.push(match f(read, *read_no.borrow(), &mut block.tags) {
            Some(str) => Some(str),
            None => None,
        });
        *read_no.get_mut() += 1;
    };
    block.segments[segment.as_index()].apply(f2);

    block.tags.insert(label.clone(), TagColumn::String(out));
}

pub(crate) fn extract_bool_tags<F>(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    mut extractor: F,
) where
    F: FnMut(&WrappedFastQRead, DemultiplexTag) -> bool,
{
    let mut values = Vec::new();
    let f = |read: &mut WrappedFastQRead, output_tag| {
        values.push(extractor(read, output_tag));
    };
    block.segments[segment.as_index()].apply_with_demultiplex_tag(f, block.output_tags.as_ref());

    block.tags.insert(label.clone(), TagColumn::Bool(values));
}

pub(crate) fn extract_bool_tags_plus_all<F, G>(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndexOrAll,
    label: &TagLabel,
    extractor_single: F,
    mut extractor_all: G,
) where
    F: FnMut(&WrappedFastQRead, DemultiplexTag) -> bool,
    G: FnMut(&Vec<WrappedFastQRead>, DemultiplexTag) -> bool,
{
    let target: Result<SegmentIndex, _> = segment.try_into();
    if let Ok(target) = target {
        // Handle single target case
        extract_bool_tags(block, target, label, extractor_single);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        let mut pos = 0;
        while let Some(molecule) = block_iter.pseudo_next() {
            let output_tag = block
                .output_tags
                .as_ref()
                .map(|x| x[pos])
                .unwrap_or_default();
            pos += 1;
            let value = extractor_all(&molecule.segments, output_tag);
            values.push(value);
        }
        block.tags.insert(label.clone(), TagColumn::Bool(values));
    }
}

pub(crate) fn extract_bool_tags_from_tag<F>(
    block: &mut FastQBlocksCombined,
    out_label: &TagLabel,
    input_label: &TagLabel,
    mut extractor: F,
) where
    F: FnMut(Option<Cow<BStr>>, DemultiplexTag) -> bool,
{
    let input_tags = block
        .tags
        .get(input_label)
        .expect("Input tag missing, validation bug");

    let mut values = Vec::new();
    for (pos, tag_value) in input_tags.iter_stringified().enumerate() {
        let output_tag = block
            .output_tags
            .as_ref()
            .map(|x| x[pos])
            .unwrap_or_default();
        values.push(extractor(tag_value, output_tag));
    }

    block
        .tags
        .insert(out_label.clone(), TagColumn::Bool(values));
}
