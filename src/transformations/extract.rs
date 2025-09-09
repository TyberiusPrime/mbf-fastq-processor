pub mod anchor;
pub mod gc_content;
pub mod iupac;
pub mod iupac_suffix;
pub mod length;
pub mod low_complexity;
pub mod low_quality_start;
pub mod low_quality_end;
pub mod mean_quality;
pub mod n_count;
pub mod poly_tail;
pub mod qualified_bases;
pub mod regex;
pub mod region;
pub mod regions;
pub mod regions_of_low_quality;
pub mod tag;

pub use anchor::Anchor;
pub use gc_content::GCContent;
pub use iupac::IUPAC;
pub use iupac_suffix::IUPACSuffix;
pub use length::Length;
pub use low_complexity::LowComplexity;
pub use low_quality_start::LowQualityStart;
pub use low_quality_end::LowQualityEnd;
pub use mean_quality::MeanQuality;
pub use n_count::NCount;
pub use poly_tail::PolyTail;
pub use qualified_bases::QualifiedBases;
pub use regex::Regex;
pub use region::Region;
pub use regions::Regions;
pub use regions_of_low_quality::RegionsOfLowQuality;

use crate::{
    config::{Target, TargetPlusAll},
    dna::TagValue,
    io,
};
use std::collections::HashMap;

pub(crate) fn extract_tags(
    block: &mut io::FastQBlocksCombined,
    target: Target,
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

    match target {
        Target::Read1 => block.read1.apply(f2),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
    };
    block.tags.as_mut().unwrap().insert(label.to_string(), out);
}

pub(crate) fn extract_numeric_tags<F>(
    target: Target,
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

    match target {
        Target::Read1 => block.read1.apply(f),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
    };

    block
        .tags
        .as_mut()
        .unwrap()
        .insert(label.to_string(), values);
}

pub(crate) fn extract_bool_tags<F>(
    block: &mut io::FastQBlocksCombined,
    target: Target,
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

    match target {
        Target::Read1 => block.read1.apply(f),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
    };

    block
        .tags
        .as_mut()
        .unwrap()
        .insert(label.to_string(), values);
}

pub(crate) fn extract_numeric_tags_plus_all<F>(
    target: TargetPlusAll,
    label: &str,
    extractor_single: F,
    mut extractor_all: impl FnMut(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> f64,
    block: &mut io::FastQBlocksCombined,
) where
    F: FnMut(&io::WrappedFastQRead) -> f64,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    if let Ok(target) = target.try_into() as Result<Target, _> {
        // Handle single target case
        extract_numeric_tags(target, label, extractor_single, block);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(
                &molecule.read1,
                molecule.read2.as_ref(),
                molecule.index1.as_ref(),
                molecule.index2.as_ref(),
            );
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
    target: TargetPlusAll,
    label: &str,
    extractor_single: F,
    mut extractor_all: G,
) where
    F: FnMut(&io::WrappedFastQRead) -> bool,
    G: FnMut(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> bool,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    if let Ok(target) = target.try_into() as Result<Target, _> {
        // Handle single target case
        extract_bool_tags(block, target, label, extractor_single);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(
                &molecule.read1,
                molecule.read2.as_ref(),
                molecule.index1.as_ref(),
                molecule.index2.as_ref(),
            );
            values.push(TagValue::Bool(value));
        }
        block
            .tags
            .as_mut()
            .unwrap()
            .insert(label.to_string(), values);
    }
}
