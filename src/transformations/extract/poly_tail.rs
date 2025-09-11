#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    Demultiplexed,
    config::{Segment, SegmentIndex, deser::base_or_dot},
    dna::Hits,
};
use anyhow::Result;
use serde_valid::Validate;

use super::super::Step;
use super::extract_tags;

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct PolyTail {
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub label: String,
    #[validate(minimum = 1)]
    pub min_length: usize,
    #[serde(deserialize_with = "base_or_dot")]
    pub base: u8,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub max_mismatch_rate: f32,
    pub max_consecutive_mismatches: usize,
}

impl PolyTail {
    #[allow(clippy::cast_precision_loss)]
    fn calc_run_length(
        seq: &[u8],
        query: u8,
        min_length: usize,
        max_mismatch_fraction: f32,
        max_consecutive_mismatches: usize,
    ) -> Option<usize> {
        if seq.len() < min_length {
            return None;
        }
        //algorithm is simple.
        // for any suffix,
        // update mismatch rate
        // if it's a match, and the mismatch rate is below the threshold,
        // and it's above the min length
        // keep the position
        // else
        // abort once even 100% matches in the remaining bases can't
        // fulfill the mismatch rate anymore.
        // or you have seen max_consecutive_mismatches
        // if no position fulfills the above, return None
        let mut matches = 0;
        let mut mismatches = 0;
        let mut last_base_pos = None;
        let seq_len = seq.len() as f32;
        let mut consecutive_mismatch_counter = 0;
        for (ii, base) in seq.iter().enumerate().rev() {
            /* dbg!(
                ii,
                base,
                *base == query,
                matches, mismatches,
                seq_len,
                mismatches as f32 / (matches + mismatches) as f32,
                (mismatches + 1) as f32 / seq_len,
                 consecutive_mismatch_counter,
                 max_consecutive_mismatches,
            );  */

            if *base == query {
                matches += 1;
                consecutive_mismatch_counter = 0;
                if seq.len() - ii >= min_length
                    && mismatches as f32 / (matches + mismatches) as f32 <= max_mismatch_fraction
                {
                    last_base_pos = Some(ii);
                }
            } else {
                mismatches += 1;
                if mismatches as f32 / seq_len > max_mismatch_fraction {
                    //dbg!("do break - mismatch rate");
                    break;
                }
                consecutive_mismatch_counter += 1;
                if consecutive_mismatch_counter >= max_consecutive_mismatches {
                    //dbg!("do break - consecutive mismatches");
                    break;
                }
            }
        }
        last_base_pos
    }
}

impl Step for PolyTail {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn tag_provides_location(&self) -> bool {
        true
    }
    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let base = self.base;
        let min_length = self.min_length;
        let max_mismatch_fraction = self.max_mismatch_rate;
        let max_consecutive_mismatches = self.max_consecutive_mismatches;
        extract_tags(
            &mut block,
            self.segment_index.as_ref().unwrap(),
            &self.label,
            |read| {
                {
                    let seq = read.seq();
                    //dbg!(std::str::from_utf8(self.name()).unwrap());

                    let last_pos = if base == b'.' {
                        let lp_a = Self::calc_run_length(
                            seq,
                            b'A',
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        );
                        let lp_c = Self::calc_run_length(
                            seq,
                            b'C',
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        );
                        let lp_g = Self::calc_run_length(
                            seq,
                            b'G',
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        );
                        let lp_t = Self::calc_run_length(
                            seq,
                            b'T',
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        );
                        let lp_n = Self::calc_run_length(
                            seq,
                            b'N',
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        );
                        //dbg!(lp_a, lp_c, lp_g, lp_t, lp_n);
                        //now I need to find the right most one that is not None
                        let mut lp = lp_a;
                        for other in [lp_g, lp_c, lp_t, lp_n] {
                            lp = match (other, lp) {
                                (None, None | Some(_)) => lp,
                                (Some(_), None) => other,
                                (Some(other_), Some(lp_)) => {
                                    if other_ < lp_ {
                                        other
                                    } else {
                                        lp
                                    }
                                }
                            };
                        }
                        lp
                    } else {
                        Self::calc_run_length(
                            seq,
                            base,
                            min_length,
                            max_mismatch_fraction,
                            max_consecutive_mismatches,
                        )
                    };
                    //dbg!(last_pos);
                    if let Some(last_pos) = last_pos {
                        Some(Hits::new(
                            last_pos,
                            seq.len() - last_pos,
                            self.segment_index.as_ref().unwrap().clone(),
                            seq[last_pos..].to_vec().into(),
                        ))
                        /* let from_end = seq.len() - last_pos;
                        self.0.seq.cut_end(from_end);
                        self.0.qual.cut_end(from_end);
                        assert!(self.0.seq.len() == self.0.qual.len()); */
                    } else {
                        None
                    }
                }
            },
        );
        (block, true)
    }
}
