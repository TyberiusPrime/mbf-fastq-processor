#![allow(clippy::struct_excessive_bools)] // can't make clippy not complain about Reports otherwise.

pub(crate) mod common;
mod inspect;
mod progress;
mod report;
mod report_base_statistics_part1;
mod report_base_statistics_part2;
mod report_count;
mod report_count_oligos;
mod report_duplicate_count;
mod report_duplicate_fragment_count;
mod report_length_distribution;
mod report_tag_histogram;

// Re-export the main structs
pub use inspect::{Inspect, PartialInspect};
pub use progress::{PartialProgress, Progress};
pub use report::{PartialReport, Report};
pub use report_base_statistics_part1::{
    _ReportBaseStatisticsPart1, Partial_ReportBaseStatisticsPart1,
};
pub use report_base_statistics_part2::{
    _ReportBaseStatisticsPart2, Partial_ReportBaseStatisticsPart2,
};
pub use report_count::{_ReportCount, Partial_ReportCount};
pub use report_count_oligos::{_ReportCountOligos, Partial_ReportCountOligos};
pub use report_duplicate_count::{_ReportDuplicateCount, Partial_ReportDuplicateCount};
pub use report_duplicate_fragment_count::{
    _ReportDuplicateFragmentCount, Partial_ReportDuplicateFragmentCount,
};
pub use report_length_distribution::{_ReportLengthDistribution, Partial_ReportLengthDistribution};
pub use report_tag_histogram::{_ReportTagHistogram, Partial_ReportTagHistogram};

// Re-export common types that might be used elsewhere
