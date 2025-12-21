//eserde false positives
#![allow(clippy::unnecessary_wraps)]
use crate::config::SegmentIndexOrAll;
use crate::config::SegmentOrAll;
use crate::transformations::prelude::*;
use anyhow::Result;

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Complexity {
    pub out_label: String,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for Complexity {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read| {
                // Calculate the number of transitions
                let mut transitions = 0;
                let seq = read.seq();
                if seq.len() <= 1 {
                    return 0.0;
                }
                for ii in 0..seq.len() - 1 {
                    if seq[ii] != seq[ii + 1] {
                        transitions += 1;
                    }
                }
                f64::from(transitions) / (seq.len() - 1) as f64
            },
            |reads| {
                let mut total_transitions = 0usize;
                let mut total_positions = 0usize;

                // Process all reads
                for read in reads {
                    let seq = read.seq();
                    for ii in 0..seq.len() - 1 {
                        if seq[ii] != seq[ii + 1] {
                            total_transitions += 1;
                        }
                    }
                    total_positions += seq.len() - 1;
                }
                if total_positions == 0 {
                    0.0
                } else {
                    total_transitions as f64 / total_positions as f64
                }
            },
            &mut block,
        );

        Ok((block, true))
    }
}
