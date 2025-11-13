// Module declarations

mod convert_quality;
mod cut_end;
mod cut_start;
mod lowercase_sequence;
mod lowercase_tag;
mod merge_reads;
mod postfix;
mod prefix;
mod rename;
mod reverse_complement;
mod reverse_complement_conditional;
mod swap;
mod swap_conditional;
mod trim_at_tag;
mod truncate;
mod uppercase_sequence;
mod uppercase_tag;

// Re-exports
pub use convert_quality::ConvertQuality;
pub use cut_end::CutEnd;
pub use cut_start::CutStart;
pub use lowercase_sequence::LowercaseSequence;
pub use lowercase_tag::LowercaseTag;
pub use merge_reads::MergeReads;
pub use postfix::Postfix;
pub use prefix::Prefix;
pub use rename::Rename;
pub use reverse_complement::ReverseComplement;
pub use reverse_complement_conditional::ReverseComplementConditional;
pub use swap::Swap;
pub use swap_conditional::SwapConditional;
pub use trim_at_tag::TrimAtTag;
pub use truncate::Truncate;
pub use uppercase_sequence::UppercaseSequence;
pub use uppercase_tag::UppercaseTag;
