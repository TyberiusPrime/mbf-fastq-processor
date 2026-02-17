// Common functionality shared by multiple tag transformations

// Individual transformation modules
pub mod concat_tags;
pub mod forget_all_tags;
pub mod forget_tag;
pub mod quantify_tag;
pub mod replace_tag_with_letter;
pub mod store_tag_in_comment;
pub mod store_tag_in_fastq;
pub mod store_tag_in_sequence;
pub mod store_tag_location_in_comment;
pub mod store_tags_in_table;

use anyhow::{Result, bail};
use bstr::{BStr, BString};
// Re-exports
pub use concat_tags::{ConcatTags, PartialConcatTags};
pub use forget_all_tags::{ForgetAllTags, PartialForgetAllTags};
pub use forget_tag::{ForgetTag, PartialForgetTag};
pub use quantify_tag::{QuantifyTag, PartialQuantifyTag};
pub use replace_tag_with_letter::{ReplaceTagWithLetter, PartialReplaceTagWithLetter};
pub use store_tag_in_comment::{StoreTagInComment, PartialStoreTagInComment};
pub use store_tag_in_fastq::{StoreTagInFastQ, PartialStoreTagInFastQ};
pub use store_tag_in_sequence::{StoreTagInSequence, PartialStoreTagInSequence};
pub use store_tag_location_in_comment::{StoreTagLocationInComment, PartialStoreTagLocationInComment};
pub use store_tags_in_table::{StoreTagsInTable, PartialStoreTagsInTable};

use crate::{
    config::{SegmentIndexOrAll},
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
        SegmentIndexOrAll::Indexed(idx) => {
            block.segments[*idx].apply_mut_with_tag(&block.tags, label, f);
        }
        SegmentIndexOrAll::All => {
            for segment_block in &mut block.segments {
                segment_block.apply_mut_with_tag(&block.tags, label, &f);
            }
        }
    }
}

// Default functions for common values
pub(crate) fn default_region_separator() -> bstr::BString {
    b"_".into()
}

pub(crate) fn default_segment_all() -> SegmentOrAll {
    SegmentOrAll("all".to_string())
}

pub(crate) fn default_comment_separator() -> u8 {
    b'|'
}

use crate::config::deser::default_comment_insert_char;

pub const DEFAULT_INITIAL_FILTER_CAPACITY: usize = 134_217_728; // 2^27. Scaleable cuckoo filters
// always need a power of 2, and we want to be north of a 'typical' danaset with 100 million reads

/// Calculate the optimal initial filter capacity based on:
/// - Configured capacity (local to the step. if provided)
/// - `InputInfo`'s `initial_filter_capacity` (if available)
/// - Demultiplexing factor (for demultiplexed filters)
///
/// # Arguments
/// * `configured_capacity` - Explicitly configured capacity (highest priority)
/// * `input_info` - Input configuration including optional `initial_filter_capacity`
/// * `demultiplex_count` - Number of demultiplex tags (for adjusting per-filter size)
/// * `debug_reproducibility` - Use small capacity for testing
///
/// # Returns
/// Capacity adjusted for demultiplexing
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[mutants::skip] // changing the base_capacity will just make things slow, not fail
pub(crate) fn calculate_filter_capacity(
    configured_capacity: Option<usize>,
    input_info: &crate::transformations::InputInfo,
    demultiplex_count: usize,
) -> usize {
    // If explicitly configured, use that value
    if let Some(capacity) = configured_capacity {
        return if demultiplex_count > 1 {
            // For demultiplexed: give each filter 1.5/n of the total
            ((capacity as f64 * 1.5) / demultiplex_count as f64).ceil() as usize
        } else {
            capacity
        };
    }

    // Use InputInfo's configured capacity if available
    let base_capacity = input_info
        .initial_filter_capacity
        .unwrap_or(DEFAULT_INITIAL_FILTER_CAPACITY);

    // Adjust for demultiplexing
    if demultiplex_count > 1 {
        ((base_capacity as f64 * 1.5) / demultiplex_count as f64).ceil() as usize
    } else {
        base_capacity
    }
}

pub(crate) fn initial_filter_elements(
    filename: &str,
    include_mapped: bool,
    include_unmapped: bool,
) -> usize {
    let bam_read_count =
        crate::io::bam_read_count_from_index(filename, include_mapped, include_unmapped);
    bam_read_count.unwrap_or(DEFAULT_INITIAL_FILTER_CAPACITY)
}

/// Format a numeric value for use in read comments, truncating floats to 4 decimal places
/// using scientific format
#[allow(clippy::cast_possible_truncation)]
pub(crate) fn format_numeric_for_comment(value: f64) -> String {
    // Check if the value is effectively an integer
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else if (1e-3..=1e6).contains(&value.abs()) {
        format!("{value:.4}")
    } else {
        format!("{value:.4e}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub(crate) fn store_tag_in_comment(
    name: &[u8],
    label: &[u8],
    tag_value: &[u8],
    comment_separator: u8,
    comment_insert_char: u8,
) -> Result<Vec<u8>> {
    if tag_value
        .iter()
        .any(|x| *x == comment_separator || *x == comment_insert_char)
    {
        bail!(
            "Tag value must not contain the comment separator ('{}'), nor the comment insert char ('{}'). Observed tag value for label '{}': '{}'",
            BString::new(vec![comment_separator]),
            BString::new(vec![comment_insert_char]),
            std::str::from_utf8(label).unwrap_or("utf-8 error"),
            BStr::new(tag_value)
        )
    }
    let insert_pos = name
        .iter()
        .position(|&x| x == comment_insert_char)
        .unwrap_or(name.len());

    let mut new_name = Vec::with_capacity(name.len() + 1 + label.len() + 1 + tag_value.len());
    new_name.extend_from_slice(&name[..insert_pos]);
    new_name.push(comment_separator);
    new_name.extend_from_slice(label);
    new_name.push(b'=');
    new_name.extend_from_slice(tag_value);
    new_name.extend_from_slice(&name[insert_pos..]);

    Ok(new_name)
}

pub fn validate_seed(seed: Option<u64>, false_positive_rate: f64) -> Result<()> {
    if false_positive_rate < 0.0 {
        bail!("False positive rate must be >= 0. Change `false_positive_rate` to a valid value.")
    } else if false_positive_rate >= 1.0 {
        bail!("False positive rate must be < 1.0 Change `false_positive_rate` to a valid value.")
    } else if false_positive_rate > 0.0 && seed.is_none() {
        bail!(
            "seed is required when false_positive_rate > 0.0 (approximate filtering). Set `seed` to 42 for example."
        );
    }
    Ok(())
}
