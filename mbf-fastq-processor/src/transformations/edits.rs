// Module declarations

mod _change_case;
mod convert_quality;
mod cut_end;
mod cut_start;
mod lowercase;
mod merge_reads;
mod postfix;
mod prefix;
mod rename;
mod reverse_complement;
mod swap;
mod trim_at_tag;
mod truncate;
mod uppercase;

// Re-exports
pub use _change_case::{_ChangeCase, CaseType};
pub use convert_quality::ConvertQuality;
pub use cut_end::CutEnd;
pub use cut_start::CutStart;
pub use lowercase::Lowercase;
pub use merge_reads::MergeReads;
pub use postfix::Postfix;
pub use prefix::Prefix;
pub use rename::Rename;
pub use reverse_complement::ReverseComplement;
pub use swap::Swap;
pub use trim_at_tag::TrimAtTag;
pub use truncate::Truncate;
pub use uppercase::Uppercase;

use crate::{io::FastQBlocksCombined, transformations::ConditionalTag};

/// Helper function to extract a boolean Vec from tags
/// Converts any tag value to its truthy representation, with optional inversion
pub(crate) fn get_bool_vec_from_tag(
    block: &FastQBlocksCombined,
    cond_tag: &ConditionalTag,
) -> Vec<bool> {
    block
        .tags
        .get(&cond_tag.tag)
        .expect("Tag not found - should have been caught in validation")
        .iter()
        .map(|tv| {
            let val = tv.truthy_val();
            if cond_tag.invert { !val } else { val }
        })
        .collect()
}
