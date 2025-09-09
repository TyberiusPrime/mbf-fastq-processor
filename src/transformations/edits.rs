// Module declarations

pub mod cut_end;
pub mod cut_start;
pub mod lowercase_sequence;
pub mod lowercase_tag;
pub mod max_len;
pub mod phred64_to33;
pub mod postfix;
pub mod prefix;
pub mod rename;
pub mod reverse_complement;
pub mod swap_r1_and_r2;
pub mod trim_at_tag;
pub mod trim_quality_end;
pub mod trim_quality_start;
pub mod uppercase_sequence;
pub mod uppercase_tag;

// Re-exports
pub use cut_end::CutEnd;
pub use cut_start::CutStart;
pub use lowercase_sequence::LowercaseSequence;
pub use lowercase_tag::LowercaseTag;
pub use max_len::MaxLen;
pub use phred64_to33::Phred64To33;
pub use postfix::Postfix;
pub use prefix::Prefix;
pub use rename::Rename;
pub use reverse_complement::ReverseComplement;
pub use swap_r1_and_r2::SwapR1AndR2;
pub use trim_at_tag::TrimAtTag;
pub use trim_quality_end::TrimQualityEnd;
pub use trim_quality_start::TrimQualityStart;
pub use uppercase_sequence::UppercaseSequence;
pub use uppercase_tag::UppercaseTag;
