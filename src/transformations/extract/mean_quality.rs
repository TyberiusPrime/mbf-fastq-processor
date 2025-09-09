#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{Demultiplexed, config::TargetPlusAll};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MeanQuality {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for MeanQuality {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let quality_scores = read.qual();
                if quality_scores.is_empty() {
                    0.0
                } else {
                    let sum: u32 = quality_scores.iter().map(|&q| u32::from(q)).sum();
                    #[allow(clippy::cast_precision_loss)]
                    {
                        f64::from(sum) / quality_scores.len() as f64
                    }
                }
            },
            |read1, read2, index1, index2| {
                let mut total_sum = 0u64;
                let mut total_length = 0usize;

                // Process read1
                let quality_scores = read1.qual();
                total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                total_length += quality_scores.len();

                // Process read2 if present
                if let Some(read2) = read2 {
                    let quality_scores = read2.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let quality_scores = index1.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let quality_scores = index2.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                if total_length == 0 {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        total_sum as f64 / total_length as f64
                    }
                }
            },
            &mut block,
        );

        (block, true)
    }
}
