// Common functionality shared by multiple tag transformations

// Individual transformation modules
pub mod forget_all_tags;
pub mod forget_tag;
pub mod numeric_to_bool;
pub mod quantify_tag;
pub mod replace_tag_with_letter;
pub mod store_tag_in_comment;
pub mod store_tag_in_fastq;
pub mod store_tag_in_sequence;
pub mod store_tag_location_in_comment;
pub mod store_tags_in_table;

use anyhow::{Result, bail};
use bstr::{BStr, BString};
use noodles::bam::bai;
use noodles::csi::binning_index::{BinningIndex, ReferenceSequence};
// Re-exports
pub use forget_all_tags::ForgetAllTags;
pub use forget_tag::ForgetTag;
pub use numeric_to_bool::NumericToBoolTag;
pub use quantify_tag::QuantifyTag;
pub use replace_tag_with_letter::ReplaceTagWithLetter;
pub use store_tag_in_comment::StoreTagInComment;
pub use store_tag_in_fastq::StoreTagInFastQ;
pub use store_tag_in_sequence::StoreTagInSequence;
pub use store_tag_location_in_comment::StoreTagLocationInComment;
pub use store_tags_in_table::StoreTagsInTable;

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    dna::TagValue,
    io,
};
use std::path::Path;

pub(crate) fn apply_in_place_wrapped_with_tag(
    segment_index: &SegmentIndexOrAll,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &TagValue),
) {
    match segment_index {
        SegmentIndexOrAll::Indexed(idx) => {
            block.segments[*idx].apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f);
        }
        SegmentIndexOrAll::All => {
            for segment_block in &mut block.segments {
                segment_block.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
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

pub(crate) fn default_comment_insert_char() -> u8 {
    b' '
}

pub(crate) fn default_replacement_letter() -> u8 {
    b'N'
}

const DEFAULT_INITIAL_FILTER_CAPACITY: usize = 10_000_000;

pub(crate) fn initial_filter_elements(filename: &str) -> usize {
    let path = Path::new(filename);

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bam"))
    {
        let candidates = [
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bam.bai");
                idx
            },
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bai");
                idx
            },
        ];

        for index_path in candidates {
            if !index_path.exists() {
                continue;
            }

            match bai::fs::read(&index_path) {
                Ok(index) => {
                    let total_reads: u128 = index
                        .reference_sequences()
                        .iter()
                        .filter_map(|reference| reference.metadata())
                        .map(|metadata| {
                            u128::from(metadata.mapped_record_count())
                                + u128::from(metadata.unmapped_record_count())
                        })
                        .sum::<u128>()
                        + u128::from(index.unplaced_unmapped_record_count().unwrap_or(0));

                    if total_reads > 0 {
                        return usize::try_from(total_reads)
                            .unwrap_or(DEFAULT_INITIAL_FILTER_CAPACITY);
                    }

                    return DEFAULT_INITIAL_FILTER_CAPACITY;
                }
                Err(error) => {
                    log::debug!("Failed to read BAM index {index_path:?} for {filename}: {error}",);
                }
            }
        }
    }

    DEFAULT_INITIAL_FILTER_CAPACITY
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
        bail!("False positive rate must be >= 0")
    } else if false_positive_rate > 0.0 && seed.is_none() {
        bail!("seed is required when false_positive_rate > 0.0 (approximate filtering)");
    }
    Ok(())
}
