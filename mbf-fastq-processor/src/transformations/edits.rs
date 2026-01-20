// Module declarations

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
mod uppercase_sequence;
mod uppercase_tag;

// Re-exports
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
pub use uppercase_sequence::UppercaseSequence;
pub use uppercase_tag::UppercaseTag; //export our struct
