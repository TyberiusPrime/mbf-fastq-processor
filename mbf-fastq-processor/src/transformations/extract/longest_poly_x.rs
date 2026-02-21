#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use crate::{config::deser::tpd_adapt_extract_base_or_dot, dna::Hits};

use super::extract_region_tags;

/// Find the longest polyX
///
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[serde(deny_unknown_fields)]
pub struct LongestPolyX {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: String,
    pub min_length: usize,
    #[tpd(with = "tpd_adapt_extract_base_or_dot")]
    pub base: u8,
    pub max_mismatch_rate: f64, //toml is f64.
    pub max_consecutive_mismatches: usize,
}

impl VerifyIn<PartialConfig> for PartialLongestPolyX {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
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
        max_mismatch_fraction: f64,
        max_consecutive_mismatches: usize,
    ) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize)> = None;

        //todo: replace this with a dynamic programming approach for better performance
        //or at least something that leverages that any run of base
        //can only start at the left most position...
        for start in 0..seq.len() - min_length {
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
                    let mismatch_ratio = mismatches as f64 / current_length as f64;
                    if mismatch_ratio <= max_mismatch_fraction {
                        best = Self::pick_better(best, Some((start, current_length)));
                    }
                }

                if (mismatches as f64) / (max_possible_length as f64) > max_mismatch_fraction {
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
        max_mismatch_fraction: f64,
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
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.min_length == 0 {
            bail!("min_length must be > 0. Set to a positive integer.");
        }
        if self.max_mismatch_rate < 0.0 || self.max_mismatch_rate >= 1.0 {
            bail!(
                "max_mismatch_rate must be in [0.0..1.0). Set to a unit scale probability >= 0 and < 1.0"
            );
        }

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self.segment;
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
