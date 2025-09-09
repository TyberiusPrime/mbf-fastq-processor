#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::TargetPlusAll, Demultiplexed};

use super::super::{ Step};
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowComplexity {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for LowComplexity {
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

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
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
            |read1, read2, index1, index2| {
                let mut total_transitions = 0usize;
                let mut total_positions = 0usize;

                // Process read1
                let seq = read1.seq();
                if seq.len() > 1 {
                    for ii in 0..seq.len() - 1 {
                        if seq[ii] != seq[ii + 1] {
                            total_transitions += 1;
                        }
                    }
                    total_positions += seq.len() - 1;
                }

                // Process read2 if present
                if let Some(read2) = read2 {
                    let seq = read2.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let seq = index1.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let seq = index2.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                if total_positions == 0 {
                    0.0
                } else {
                    total_transitions as f64 / total_positions as f64
                }
            },
            &mut block,
        );

        (block, true)
    }
}