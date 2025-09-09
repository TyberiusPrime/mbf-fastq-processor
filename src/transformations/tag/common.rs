#![allow(clippy::unnecessary_wraps)] //eserde false positives
use std::collections::HashMap;

use crate::{
    config::{Target, TargetPlusAll},
    dna::TagValue,
    io,
};

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
    extractor: F,
    block: &mut io::FastQBlocksCombined,
) where
    F: Fn(&io::WrappedFastQRead) -> f64,
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

pub(crate) fn extract_numeric_tags_plus_all<F>(
    target: TargetPlusAll,
    label: &str,
    extractor_single: F,
    extractor_all: impl Fn(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> f64,
    block: &mut io::FastQBlocksCombined,
) where
    F: Fn(&io::WrappedFastQRead) -> f64,
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

pub(crate) fn apply_in_place_wrapped_with_tag(
    target: TargetPlusAll,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &TagValue),
) {
    match target {
        TargetPlusAll::Read1 => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f);
        }

        TargetPlusAll::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::All => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            if let Some(read2) = &mut block.read2 {
                read2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index1) = &mut block.index1 {
                index1.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index2) = &mut block.index2 {
                index2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
        }
    }
}

// Default functions for common values
pub(crate) fn default_region_separator() -> bstr::BString {
    b"-".into()
}

pub(crate) fn default_target_read1() -> TargetPlusAll {
    TargetPlusAll::Read1
}

pub(crate) fn default_comment_separator() -> u8 {
    b'|'
}

pub(crate) fn default_comment_insert_char() -> u8 {
    b' '
}

pub(crate) fn default_replacement_letter() -> u8 {
    b'N'
}

/// Format a numeric value for use in read comments, truncating floats to 4 decimal places
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn format_numeric_for_comment(value: f64) -> String {
    // Check if the value is effectively an integer
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value:.4}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub(crate) fn store_tag_in_comment(
    read: &mut crate::io::WrappedFastQReadMut,
    label: &[u8],
    tag_value: &[u8],
    comment_separator: u8,
    comment_insert_char: u8,
) {
    let name = read.name();
    assert!(
        !tag_value.iter().any(|x| *x == comment_separator),
        "Tag value for {} contains the comment separator '{}'. This would break the read name. Please change the tag value or the comment separator.",
        std::str::from_utf8(label).unwrap_or("utf-8 error"),
        comment_separator as char
    );
    let insert_pos = read
        .name()
        .iter()
        .position(|&x| x == comment_insert_char)
        .unwrap_or(read.name().len());

    let mut new_name =
        Vec::with_capacity(read.name().len() + 1 + label.len() + 1 + tag_value.len());
    new_name.extend_from_slice(&name[..insert_pos]);
    new_name.push(comment_separator);
    new_name.extend_from_slice(label);
    new_name.push(b'=');
    new_name.extend_from_slice(tag_value);
    new_name.extend_from_slice(&name[insert_pos..]);

    read.replace_name(new_name);
}