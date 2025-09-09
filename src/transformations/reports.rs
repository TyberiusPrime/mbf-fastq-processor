#![allow(clippy::struct_excessive_bools)] // can't make clippy not complain about Reports otherwise.

pub mod common;
pub mod progress;
pub mod report;
pub mod report_count;
pub mod report_length_distribution;
pub mod report_duplicate_count;
pub mod report_duplicate_fragment_count;
pub mod report_base_statistics_part1;
pub mod report_base_statistics_part2;
pub mod report_count_oligos;
pub mod inspect;

// Re-export the main structs
pub use progress::Progress;
pub use report::Report;
pub use report_count::_ReportCount;
pub use report_length_distribution::_ReportLengthDistribution;
pub use report_duplicate_count::_ReportDuplicateCount;
pub use report_duplicate_fragment_count::_ReportDuplicateFragmentCount;
pub use report_base_statistics_part1::_ReportBaseStatisticsPart1;
pub use report_base_statistics_part2::_ReportBaseStatisticsPart2;
pub use report_count_oligos::_ReportCountOligos;
pub use inspect::Inspect;

// Re-export common types that might be used elsewhere
