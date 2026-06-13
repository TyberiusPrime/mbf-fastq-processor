use super::extract_region_tags_from_seq;
use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_extract_base_or_dot;

/// Extract ends that are homo-polymers into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct PolyTail {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: TagLabel,
    pub min_length: usize,
    #[tpd(with = "tpd_adapt_extract_base_or_dot")]
    pub base: u8,
    pub max_mismatch_rate: f64,
    pub max_consecutive_mismatches: usize,
    /// Use fastp's polyG trimming algorithm (requires base = 'G').
    /// Applies fastp's fixed constants: 1 mismatch allowed per 8 bases, max 5 total mismatches.
    pub fastp_mode: Option<bool>,
}

impl VerifyIn<PartialConfig> for PartialPolyTail {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.min_length.verify(|v| {
            if *v < 2 {
                Err(ValidationFailure::new(
                    "Invalid value. Must be >= 2",
                    Some("Change to a positive integer larger than 1."),
                ))
            } else {
                Ok(())
            }
        });
        self.max_mismatch_rate.verify(|v| {
            if *v < 0.0 || *v >= 1.0 {
                Err(ValidationFailure::new(
                    "Invalid value. Must be in [0.0..1.0)",
                    Some("Set a valid value >= 0 and < 1.0."),
                ))
            } else {
                Ok(())
            }
        });
        let fastp_enabled = self.fastp_mode.as_ref().copied().flatten().unwrap_or(false);
        if fastp_enabled {
            if let Some(base) = self.base.as_ref()
                && *base != b'G'
            {
                let spans = vec![
                    (
                        self.base.span(),
                        "This must be 'G' when fastp_mode is enabled".to_string(),
                    ),
                    (
                        self.fastp_mode.span(),
                        "fastp_mode is enabled here".to_string(),
                    ),
                ];
                self.fastp_mode.state = TomlValueState::Custom { spans };
                self.fastp_mode.help = Some("Set base = 'G' or disable fastp_mode.".to_string());
            }
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialPolyTail> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            // Record the segment this location tag lives on, so a conditional
            // `Swap` on that segment can forget it (a per-read swap would split it
            // across two segments).
            let segment = inner
                .segment
                .as_ref()
                .and_then(|m| m.as_ref_post())
                .copied();
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Location)
                    .map(|dt| match segment {
                        Some(seg) => dt.with_segment(seg),
                        None => dt,
                    }),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for PolyTail {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let base = self.base;
        let min_length = self.min_length;
        let max_mismatch_fraction = self.max_mismatch_rate;
        let max_consecutive_mismatches = self.max_consecutive_mismatches;
        let fastp_mode = self.fastp_mode.unwrap_or(false);
        extract_region_tags_from_seq(&mut block, self.segment, &self.out_label, |read_seq| {
            let last_pos = if fastp_mode {
                find_poly_tail_fastp(read_seq, min_length)
            } else {
                find_poly_tail(
                    read_seq,
                    base,
                    min_length,
                    max_mismatch_fraction,
                    max_consecutive_mismatches,
                )
            };

            last_pos.map(|last_pos| last_pos as u32..read_seq.len() as u32)
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

#[expect(
    clippy::cast_precision_loss,
    reason = "seq.len() will fit into an f64 for realistic values"
)]
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

fn find_poly_tail_fastp(seq: &[u8], min_length: usize) -> Option<usize> {
    // Replicates fastp's trimPolyG algorithm:
    //   allow 1 mismatch per ALLOW_ONE_MISMATCH_FOR_EACH bases (integer division),
    //   hard cap of MAX_MISMATCH total mismatches,
    //   compareReq = min_length.
    const ALLOW_ONE_MISMATCH_FOR_EACH: usize = 8;
    const MAX_MISMATCH: usize = 5;

    let rlen = seq.len();
    if rlen < min_length {
        return None;
    }

    let mut mismatch: usize = 0;
    let mut first_g_pos = rlen - 1;
    let mut i: usize = 0;
    while i < rlen {
        let pos = rlen - i - 1;
        if seq[pos] != b'G' {
            mismatch += 1;
        } else {
            first_g_pos = pos;
        }
        let allowed_mismatch = (i + 1) / ALLOW_ONE_MISMATCH_FOR_EACH;
        if mismatch > MAX_MISMATCH || (mismatch > allowed_mismatch && i >= min_length - 1) {
            break;
        }
        i += 1;
    }

    if i >= min_length {
        Some(first_g_pos)
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use super::{calc_run_length, find_poly_tail, find_poly_tail_fastp};

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

    #[test]
    fn test_find_poly_tail_fastp() {
        // seq1: complex mix of Gs and non-Gs at the end — regular mode trims but fastp leaves intact
        // because the loop stops at i=9 < min_length=10 due to mismatch rate exceeded.
        assert_eq!(
            find_poly_tail_fastp(
                b"GTCCGTGTCCCTTGCGTAGATGAGTGTTTCATGAGTGTGGGTGTGAGTGTGAATGTGTCTAGATGAGAATGGGGGGGGGGGGGGGGGGGGGTTAGGGGGG",
                10
            ),
            None
        );

        // seq2: fastp stops at the 2nd T from the right (pos 87 is last T kept, tail starts at 88).
        // Regular mode would extend further into the G-rich region.
        assert_eq!(
            find_poly_tail_fastp(
                b"CTTTTGCTGTTTTTTTTTTTTTTTTTTTTTGGGGTAGGGGGGGGAAGCTTTTGATGCAGTTGCCGGGGTGCGACGGAACGGACGGGGTGGGGGGGGGTGG",
                10
            ),
            Some(88)
        );

        // seq3: fastp trims the 9-G run at the end (tail starts at 91).
        // Regular mode with lenient parameters leaves intact because the non-G chars before
        // the tail interrupt the count before min_length is reached.
        assert_eq!(
            find_poly_tail_fastp(
                b"AGGAGCAGCGGGTGCGGAGTAGGCGGGAGCAGCGGGTGCGGAGTAGGCTGGGGCAGCTGGAGCAGAGTAGGCCTGGGCAGCGGGAGCGGCTGGGGGGGGG",
                10
            ),
            Some(91)
        );

        // Pure G run: tail is the whole sequence.
        assert_eq!(find_poly_tail_fastp(b"GGGGGGGGGG", 10), Some(0));

        // One short of min_length: not trimmed.
        assert_eq!(find_poly_tail_fastp(b"GGGGGGGGG", 10), None);

        // Single non-G before sufficient Gs: within the 1-per-8 allowance, tail still detected.
        // 15 Gs + 1 T + 1 G = 17 chars; scanning from right: i=0 G, ..., i=14 G, i=15 T (mismatch=1,
        // allowed=2, no break), i=16 G (firstGPos=0). i=16 >= 10, return Some(0).
        assert_eq!(find_poly_tail_fastp(b"GGGGGGGGGGGGGGGTG", 10), Some(0));

        // 4 leading Ts followed by 13 Gs: the T run triggers the break but i=14>=10, so
        // the 13-G tail (starting at pos 4) is still detected.
        assert_eq!(find_poly_tail_fastp(b"TTTTGGGGGGGGGGGGG", 10), Some(4));
    }
}
