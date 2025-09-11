// Common functionality shared by multiple tag transformations

// Individual transformation modules
pub mod quantify_tag;
pub mod remove_tag;
pub mod replace_tag_with_letter;
pub mod store_tag_in_comment;
pub mod store_tag_in_sequence;
pub mod store_tag_location_in_comment;
pub mod store_tags_in_table;

// Re-exports
pub use quantify_tag::QuantifyTag;
pub use remove_tag::RemoveTag;
pub use replace_tag_with_letter::ReplaceTagWithLetter;
pub use store_tag_in_comment::StoreTagInComment;
pub use store_tag_in_sequence::StoreTagInSequence;
pub use store_tag_location_in_comment::StoreTaglocationInComment;
pub use store_tags_in_table::StoreTagsInTable;

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    dna::TagValue,
    io,
};

pub(crate) fn apply_in_place_wrapped_with_tag(
    segment_index: &SegmentIndexOrAll,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &TagValue),
) {
    match segment_index {
        SegmentIndexOrAll::Indexed(idx, _name) => {
            block.segments[*idx].apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f);
        }
        SegmentIndexOrAll::All => {
            for segment_block in block.segments.iter_mut() {
                segment_block.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
        }
    }
}

// Default functions for common values
pub(crate) fn default_region_separator() -> bstr::BString {
    b"-".into()
}

pub(crate) fn default_segment_all() -> SegmentOrAll {
    SegmentOrAll("all".to_string())
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
