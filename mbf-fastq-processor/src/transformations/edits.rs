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
pub use convert_quality::{ConvertQuality, PartialConvertQuality};
pub use cut_end::{CutEnd, PartialCutEnd};
pub use cut_start::{CutStart, PartialCutStart};
pub use lowercase::{Lowercase, PartialLowercase};
pub use merge_reads::{MergeReads, PartialMergeReads};
pub use postfix::{Postfix, PartialPostfix};
pub use prefix::{Prefix, PartialPrefix};
pub use rename::{Rename, PartialRename};
pub use reverse_complement::{ReverseComplement, PartialReverseComplement};
pub use swap::{Swap, PartialSwap};
pub use trim_at_tag::{TrimAtTag, PartialTrimAtTag};
pub use truncate::{Truncate, PartialTruncate};
pub use uppercase::{Uppercase, PartialUppercase};

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
