use std::{borrow::Cow, cell::RefCell, ops::Range};

use bstr::{BStr, BString};
use fastqrab_dna::dna::TagColumn;
use fastqrab_io::blocks::Molecule;
use indexmap::IndexMap;
use stringpod::CrossPods;

use super::prelude::DemultiplexTag;
use fastqrab_config::{
    TagLabel,
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

pub(crate) fn extract_region_tags_from_seq(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&BStr) -> Option<Range<u32>>,
) {
    let mut col = LocationColumn::new();
    for seq in block.segments[segment.as_index()].seq_quals.iter_seq() {
        match f(seq) {
            Some(draft) => col.push_single(draft.location, &draft.sequence),
            None => col.push_none(),
        }
    }

    block.tags.insert(label.clone(), TagColumn::Location(col));
}

pub(crate) fn extract_region_tags_from_both(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&BStr, &BStr) -> Option<Range<u32>>,
) {
    let mut col = LocationColumn::new();
    for read in block.segments[segment.as_index()].seq_quals.iter() {
        match f(read.seq, read.qual) {
            Some(draft) => col.push_single(draft.location, &draft.sequence),
            None => col.push_none(),
        }
    }

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
    f: impl Fn(&BStr, usize, &IndexMap<TagLabel, TagColumn>) -> Option<Range<u32>>,
) {
    let mut col = LocationColumn::new();

    let read_no = block.first_read_sequential_number;
    for (ii, seq) in block.segments[segment.as_index()]
        .seq_quals
        .iter_seq()
        .enumerate()
    {
        match f(seq, read_no + ii, &mut block.tags) {
            Some(draft) => col.push_single(draft.location, &draft.sequence),
            None => col.push_none(),
        }
    }

    block.tags.insert(label.clone(), TagColumn::Location(col));
}

pub(crate) fn extract_string_tags_using_tags(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    f: impl Fn(&BStr, usize, &IndexMap<TagLabel, TagColumn>) -> Option<BString>,
) {
    let mut out = Vec::new();
    let read_no = block.first_read_sequential_number;

    let read_no = block.first_read_sequential_number;
    for (ii, seq) in block.segments[segment.as_index()]
        .seq_quals
        .iter_seq()
        .enumerate()
    {
        out.push(match f(seq, read_no + ii, &mut block.tags) {
            Some(str) => Some(str),
            None => None,
        });
    }

    block.tags.insert(label.clone(), TagColumn::String(out));
}

pub(crate) fn extract_bool_tags<F>(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndex,
    label: &TagLabel,
    mut extractor: F,
) where
    F: FnMut(&fastqrab_io::blocks::FastQRead, DemultiplexTag) -> bool,
{
    let mut values = Vec::new();
    for (idx, read) in block.segments[segment.as_index()].iter().enumerate() {
        let output_tag = block
            .output_tags
            .as_ref()
            .map(|x| x[idx])
            .unwrap_or_default();
        values.push(extractor(&read, output_tag));
    }

    block.tags.insert(label.clone(), TagColumn::Bool(values));
}

pub(crate) fn extract_bool_tags_plus_all<F, G>(
    block: &mut FastQBlocksCombined,
    segment: SegmentIndexOrAll,
    label: &TagLabel,
    extractor_single: F,
    mut extractor_all: G,
) where
    F: FnMut(&fastqrab_io::blocks::FastQRead, DemultiplexTag) -> bool,
    G: FnMut(&Molecule<'_>, DemultiplexTag) -> bool,
{
    let target: Result<SegmentIndex, _> = segment.try_into();
    if let Ok(target) = target {
        // Handle single target case
        extract_bool_tags(block, target, label, extractor_single);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        for (idx, molecule) in block.molecules().enumerate() {
            let output_tag = block
                .output_tags
                .as_ref()
                .map(|x| x[idx])
                .unwrap_or_default();
            let value = extractor_all(&molecule, output_tag);
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
