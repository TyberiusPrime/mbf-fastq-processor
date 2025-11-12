mod anchor;
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

pub use anchor::Anchor;
use bstr::BString;
pub use iupac::IUPAC;
pub use iupac_suffix::IUPACSuffix;
pub use iupac_with_indel::IUPACWithIndel;
pub use longest_poly_x::LongestPolyX;
pub use low_quality_end::LowQualityEnd;
pub use low_quality_start::LowQualityStart;
pub use poly_tail::PolyTail;
pub use regex::Regex;
pub use region::Region;
pub use regions::Regions;
pub use regions_of_low_quality::RegionsOfLowQuality;

use crate::{
    config::{SegmentIndex, SegmentIndexOrAll},
    dna::TagValue,
    io,
};

use super::prelude::DemultiplexTag;

pub(crate) fn extract_region_tags(
    block: &mut io::FastQBlocksCombined,
    segment: SegmentIndex,
    label: &str,
    f: impl Fn(&mut io::WrappedFastQRead) -> Option<crate::dna::Hits>,
) {
    let mut out = Vec::new();

    let f2 = |read: &mut io::WrappedFastQRead| {
        out.push(match f(read) {
            Some(hits) => TagValue::Location(hits),
            None => TagValue::Missing,
        });
    };
    block.segments[segment.get_index()].apply(f2);

    block.tags.insert(label.to_string(), out);
}

pub(crate) fn extract_string_tags(
    block: &mut io::FastQBlocksCombined,
    segment: SegmentIndex,
    label: &str,
    f: impl Fn(&mut io::WrappedFastQRead) -> Option<BString>,
) {
    let mut out = Vec::new();

    let f2 = |read: &mut io::WrappedFastQRead| {
        out.push(match f(read) {
            Some(hits) => TagValue::String(hits),
            None => TagValue::Missing,
        });
    };
    block.segments[segment.get_index()].apply(f2);

    block.tags.insert(label.to_string(), out);
}

pub(crate) fn extract_bool_tags<F>(
    block: &mut io::FastQBlocksCombined,
    segment: SegmentIndex,
    label: &str,
    mut extractor: F,
) where
    F: FnMut(&io::WrappedFastQRead, DemultiplexTag) -> bool,
{

    let mut values = Vec::new();
    let f = |read: &mut io::WrappedFastQRead, output_tag| {
        values.push(TagValue::Bool(extractor(read, output_tag)));
    };
    block.segments[segment.get_index()].apply_with_demultiplex_tag(f, block.output_tags.as_ref());

    block
        .tags
        .insert(label.to_string(), values);
}

pub(crate) fn extract_bool_tags_plus_all<F, G>(
    block: &mut io::FastQBlocksCombined,
    segment: SegmentIndexOrAll,
    label: &str,
    extractor_single: F,
    mut extractor_all: G,
) where
    F: FnMut(&io::WrappedFastQRead, DemultiplexTag) -> bool,
    G: FnMut(&Vec<io::WrappedFastQRead>, DemultiplexTag) -> bool,
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
            values.push(TagValue::Bool(value));
        }
        block
            .tags
            .insert(label.to_string(), values);
    }
}

pub(crate) fn extract_bool_tags_from_tag<F>(
    block: &mut io::FastQBlocksCombined,
    label: &str,
    input_label: &str,
    mut extractor: F,
) where
    F: FnMut(&TagValue, DemultiplexTag) -> bool,
{
  

    let input_tags = block
        .tags
        .get(input_label)
        .expect("Input tag missing, validation bug");

    let mut values = Vec::new();
    for (pos, tag_value) in input_tags.iter().enumerate() {
        let output_tag = block
            .output_tags
            .as_ref()
            .map(|x| x[pos])
            .unwrap_or_default();
        values.push(TagValue::Bool(extractor(tag_value, output_tag)));
    }

    block
        .tags
        .insert(label.to_string(), values);
}
