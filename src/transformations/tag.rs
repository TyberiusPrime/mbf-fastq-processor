#![allow(clippy::unnecessary_wraps)] //eserde false positives

// Common functionality shared by multiple tag transformations
pub mod common;

// Individual transformation modules
pub mod extract_iupac;
pub mod extract_regex;
pub mod extract_anchor;
pub mod extract_poly_tail;
pub mod extract_iupac_suffix;
pub mod filter_by_tag;
pub mod trim_at_tag;
pub mod extract_region;
pub mod extract_regions;
pub mod store_tag_in_sequence;
pub mod store_tag_in_comment;
pub mod store_tag_location_in_comment;
pub mod extract_length;
pub mod extract_mean_quality;
pub mod extract_gc_content;
pub mod extract_n_count;
pub mod extract_low_complexity;
pub mod extract_qualified_bases;
pub mod remove_tag;
pub mod store_tags_in_table;
pub mod quantify_tag;
pub mod extract_regions_of_low_quality;
pub mod replace_tag_with_letter;

// Re-exports
pub use extract_iupac::ExtractIUPAC;
pub use extract_regex::ExtractRegex;
pub use extract_anchor::ExtractAnchor;
pub use extract_poly_tail::ExtractPolyTail;
pub use extract_iupac_suffix::ExtractIUPACSuffix;
pub use filter_by_tag::FilterByTag;
pub use trim_at_tag::TrimAtTag;
pub use extract_region::ExtractRegion;
pub use extract_regions::ExtractRegions;
pub use store_tag_in_sequence::StoreTagInSequence;
pub use store_tag_in_comment::StoreTagInComment;
pub use store_tag_location_in_comment::StoreTaglocationInComment;
pub use extract_length::ExtractLength;
pub use extract_mean_quality::ExtractMeanQuality;
pub use extract_gc_content::ExtractGCContent;
pub use extract_n_count::ExtractNCount;
pub use extract_low_complexity::ExtractLowComplexity;
pub use extract_qualified_bases::ExtractQualifiedBases;
pub use remove_tag::RemoveTag;
pub use store_tags_in_table::StoreTagsInTable;
pub use quantify_tag::QuantifyTag;
pub use extract_regions_of_low_quality::ExtractRegionsOfLowQuality;
pub use replace_tag_with_letter::ReplaceTagWithLetter;