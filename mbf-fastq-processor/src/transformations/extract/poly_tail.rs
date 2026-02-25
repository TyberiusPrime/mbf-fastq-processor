#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::{config::deser::tpd_adapt_extract_base_or_dot, transformations::prelude::*};

use super::extract_region_tags;
use crate::dna::Hits;

/// Extract ends that are homo-polymers into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct PolyTail {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: String,
    //#[validate(minimum = 1)] todo
    pub min_length: usize,
    #[tpd(with = "tpd_adapt_extract_base_or_dot")]
    pub base: u8,
    //#[validate(minimum = 0.)]// todo
    //#[validate(maximum = 1.)] //todo
    pub max_mismatch_rate: f64,
    pub max_consecutive_mismatches: usize,
}

impl VerifyIn<PartialConfig> for PartialPolyTail {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.min_length.verify(|v| {
            if *v < 2 {
                Err(ValidationFailure::new(
                    "min_length must be >= 2",
                    Some("Change to a positive integer larger than 1"),
                ))
            } else {
                Ok(())
            }
        });
        self.max_mismatch_rate.verify(|v| {
            if *v < 0.0 || *v >= 1.0 {
                Err(ValidationFailure::new(
                    "max_mismatch_rate must be in [0.0..1.0)",
                    Some("Set a valid value >= 0 and < 1.0"),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl Step for PolyTail {
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
        let base = self.base;
        let min_length = self.min_length;
        let max_mismatch_fraction = self.max_mismatch_rate;
        let max_consecutive_mismatches = self.max_consecutive_mismatches;
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
            {
                let seq = read.seq();
                //dbg!(std::str::from_utf8(self.name()).unwrap());
                //
                let last_pos = find_poly_tail(
                    seq,
                    base,
                    min_length,
                    max_mismatch_fraction,
                    max_consecutive_mismatches,
                );

                //dbg!(last_pos);
                last_pos.map(|last_pos| {
                    Hits::new(
                        last_pos,
                        seq.len() - last_pos,
                        self.segment,
                        seq[last_pos..].to_vec().into(),
                    )
                })
            }
        });
        Ok((block, true))
    }
}

fn find_poly_tail(
    seq: &[u8],
    base: u8,
    min_length: usize,
    max_mismatch_fraction: f64,
    max_consecutive_mismatches: usize,
) -> Option<usize> {
    if base == b'.' {
        let lp_a = calc_run_length(
            seq,
            b'A',
            min_length,
            max_mismatch_fraction,
            max_consecutive_mismatches,
        );
        let lp_c = calc_run_length(
            seq,
            b'C',
            min_length,
            max_mismatch_fraction,
            max_consecutive_mismatches,
        );
        let lp_g = calc_run_length(
            seq,
            b'G',
            min_length,
            max_mismatch_fraction,
            max_consecutive_mismatches,
        );
        let lp_t = calc_run_length(
            seq,
            b'T',
            min_length,
            max_mismatch_fraction,
            max_consecutive_mismatches,
        );
        let lp_n = calc_run_length(
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
                    //remember it's last pos, so Smaller is longer
                    if other_ < lp_ { other } else { lp }
                }
            };
        }
        lp
    } else {
        calc_run_length(
            seq,
            base,
            min_length,
            max_mismatch_fraction,
            max_consecutive_mismatches,
        )
    }
}

#[allow(clippy::cast_precision_loss)]
fn calc_run_length(
    seq: &[u8],
    query: u8,
    min_length: usize,
    max_mismatch_fraction: f64,
    max_consecutive_mismatches: usize,
) -> Option<usize> {
    if seq.len() < min_length {
        //optimization. mutation analysis will flag it for being useless.
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
    let seq_len = seq.len() as f64;
    let mut consecutive_mismatch_counter = 0;
    for (ii, base) in seq.iter().enumerate().rev() {
        //  dbg!(
        //     ii,
        //     base,
        //     *base == query,
        //     matches, mismatches,
        //     seq_len,
        //     mismatches as f64 / (matches + mismatches) as f64,
        //     (mismatches + 1) as f64 / seq_len,
        //      consecutive_mismatch_counter,
        //      max_consecutive_mismatches,
        // );

        if *base == query {
            matches += 1;
            consecutive_mismatch_counter = 0;
            let local_rate = f64::from(mismatches) / f64::from(matches + mismatches);
            if seq.len() - ii >= min_length && local_rate <= max_mismatch_fraction {
                last_base_pos = Some(ii);
            }
        } else {
            mismatches += 1;
            if f64::from(mismatches) / seq_len > max_mismatch_fraction {
                //dbg!("do break - mismatch rate");
                break;
            }
            consecutive_mismatch_counter += 1;
            if consecutive_mismatch_counter > max_consecutive_mismatches {
                //dbg!("do break - consecutive mismatches");
                break;
            }
        }
    }
    last_base_pos
}

#[cfg(test)]
mod test {
    use super::{calc_run_length, find_poly_tail};

    #[test]
    fn test_calc_run_length() {
        assert_eq!(
            calc_run_length(
                b"AGTCAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                b'A',
                10,
                0.1,
                2
            ),
            Some(4)
        );
        assert_eq!(calc_run_length(b"AAAAAA", b'A', 3, 0.1, 2), Some(0));
        assert_eq!(calc_run_length(b"AAATAA", b'A', 3, 0.34, 2), Some(0));
        assert_eq!(calc_run_length(b"AAATAA", b'A', 3, 0.0, 0), None);
        assert_eq!(
            calc_run_length(b"AAATAA", b'A', 3, 1.0 / 6.0 - 0.001, 2),
            None
        );
        assert_eq!(calc_run_length(b"ATTTTTT", b'A', 30, 0.108_123, 20), None);
    }

    #[test]
    fn test_find_poly_tail() {
        assert_eq!(calc_run_length(b"AAAAAAAAACCC", b'A', 3, 0.4, 3), Some(0));
        assert_eq!(find_poly_tail(b"AAATAACCCCCC", b'.', 3, 0.2, 2), Some(6));
        assert_eq!(find_poly_tail(b"AAAAAAAAACCC", b'.', 3, 0.4, 3), Some(0));
        assert_eq!(find_poly_tail(b"GGGGGGGGGCCC", b'.', 3, 0.4, 3), Some(0));
        assert_eq!(find_poly_tail(b"CCCCCCCCCAAA", b'.', 3, 0.4, 3), Some(0));
    }
}
