#![allow(clippy::unnecessary_wraps)]
use crate::config::SegmentIndexOrAll;
//eserde false positives
use crate::{Demultiplexed, config::SegmentOrAll};
use anyhow::Result;

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MeanQuality {
    pub label: String,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for MeanQuality {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment_index.as_ref().unwrap(),
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
            |reads| {
                let mut total_sum = 0u64;
                let mut total_length = 0usize;

                for read in reads {
                    let quality_scores = read.qual();
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

        Ok((block, true))
    }
}
