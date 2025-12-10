#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use serde_valid::Validate;

use crate::{
    config::{Segment, SegmentIndex, deser::base_or_dot},
    dna::Hits,
};

use super::extract_region_tags;

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LongestPolyX {
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,
    #[validate(minimum = 1)]
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub max_mismatch_rate: f32,
    pub max_consecutive_mismatches: usize,
}

impl LongestPolyX {
    fn pick_better(
        current: Option<(usize, usize)>,
        candidate: Option<(usize, usize)>,
    ) -> Option<(usize, usize)> {
        match (current, candidate) {
            (None, None) => None,
            (Some(existing), None) => Some(existing),
            (None, Some(new_candidate)) => Some(new_candidate),
            (Some(existing), Some(new_candidate)) => {
                if new_candidate.1 > existing.1
                    || (new_candidate.1 == existing.1 && new_candidate.0 < existing.0)
                {
                    Some(new_candidate)
                } else {
                    Some(existing)
                }
            }
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn find_best_for_base(
        seq: &[u8],
        base: u8,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_consecutive_mismatches: usize,
    ) -> Option<(usize, usize)> {
        if seq.len() < min_length {
            return None;
        }

        let mut best: Option<(usize, usize)> = None;

        for start in 0..seq.len() {
            if seq.len() - start < min_length {
                break;
            }

            let mut mismatches = 0;
            let mut consecutive_mismatches = 0;
            let max_possible_length = seq.len() - start;

            for (end, symbol) in seq.iter().enumerate().skip(start) {
                if *symbol == base {
                    consecutive_mismatches = 0;
                } else {
                    mismatches += 1;
                    consecutive_mismatches += 1;
                    if consecutive_mismatches >= max_consecutive_mismatches {
                        break;
                    }
                }

                let current_length = end - start + 1;

                if current_length >= min_length {
                    let mismatch_ratio = mismatches as f32 / current_length as f32;
                    if mismatch_ratio <= max_mismatch_fraction {
                        best = Self::pick_better(best, Some((start, current_length)));
                    }
                }

                if (mismatches as f32) / (max_possible_length as f32) > max_mismatch_fraction {
                    break;
                }
            }
        }

        best
    }

    fn find_best(
        seq: &[u8],
        base: u8,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_consecutive_mismatches: usize,
    ) -> Option<(usize, usize)> {
        if base == b'.' {
            let mut best = None;
            for candidate_base in [b'A', b'C', b'G', b'T'] {
                let candidate = Self::find_best_for_base(
                    seq,
                    candidate_base,
                    min_length,
                    max_mismatch_fraction,
                    max_consecutive_mismatches,
                );
                best = Self::pick_better(best, candidate);
            }
            best
        } else {
            Self::find_best_for_base(
                seq,
                base,
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            )
        }
    }
}

impl Step for LongestPolyX {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self
            .segment_index
            .expect("segment_index must be set during initialization");
        let min_length = self.min_length;
        let base = self.base;
        let max_mismatch_fraction = self.max_mismatch_rate;
        let max_consecutive_mismatches = self.max_consecutive_mismatches;

        extract_region_tags(&mut block, segment_index, &self.out_label, move |read| {
            let seq = read.seq();
            Self::find_best(
                seq,
                base,
                min_length,
                max_mismatch_fraction,
                max_consecutive_mismatches,
            )
            .map(|(start, len)| {
                Hits::new(
                    start,
                    len,
                    segment_index,
                    seq[start..start + len].to_vec().into(),
                )
            })
        });
        Ok((block, true))
    }
}
