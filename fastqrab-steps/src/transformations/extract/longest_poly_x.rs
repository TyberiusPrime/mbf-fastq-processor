use super::extract_region_tags_from_seq;
use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_extract_base_or_dot;

/// Find the longest polyX
///
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct LongestPolyX {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: TagLabel,
    pub min_length: usize,
    #[tpd(with = "tpd_adapt_extract_base_or_dot")]
    pub base: u8,
    pub max_mismatch_rate: f64, //toml is f64.
    pub max_consecutive_mismatches: usize,
}

impl VerifyIn<PartialConfig> for PartialLongestPolyX {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.segment.validate_segment(parent);
        self.min_length.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "min_length must be > 0",
                    Some("Set to a positive integer"),
                ))
            } else {
                Ok(())
            }
        });
        self.max_mismatch_rate.verify(|v| {
            if *v < 0.0 || *v >= 1.0 {
                Err(ValidationFailure::new(
                    "max_mismatch_rate must be in [0.0..1.0)",
                    Some("Set to a unit scale probability >= 0 and < 1.0"),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl LongestPolyX {
    fn pick_better(
        current: Option<(usize, usize)>,
        candidate: Option<(usize, usize)>,
    ) -> Option<(usize, usize)> {
        match (current, candidate) {
            (None, None) => None, // cov:excl-line shouldn't happen, should it?
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

    /// O(n) algorithm: compute prefix sums and barrier runs for all bases in a
    /// single pass over the sequence, then find the longest valid subarray per
    /// barrier-free segment using a monotone-stack approach.
    fn find_best(
        seq: &[u8],
        base: u8,
        min_length: usize,
        max_mismatch_fraction: f64,
        max_consecutive_mismatches: usize,
    ) -> Option<(usize, usize)> {
        let n = seq.len();
        if n < min_length {
            return None;
        }

        // max_consecutive_mismatches == 0 means only exact runs of the base are
        // valid (the first mismatch terminates the run).
        if max_consecutive_mismatches == 0 {
            return Self::longest_exact_run(seq, base, min_length);
        }

        let check_bases: &[u8] = if base == b'.' {
            b"ACGT"
        } else {
            std::slice::from_ref(&base)
        };
        let num = check_bases.len();

        // Weighted prefix sums: match contributes +rate, mismatch contributes
        // +(rate-1). A subarray [l,r] satisfies the mismatch-rate constraint
        // iff prefix[r+1] - prefix[l] >= 0.
        let match_w = max_mismatch_fraction;
        let mis_w = max_mismatch_fraction - 1.0;
        let max_consec = max_consecutive_mismatches;

        let mut prefixes: Vec<Vec<f64>> = (0..num)
            .map(|_| {
                let mut v = Vec::with_capacity(n + 1);
                v.push(0.0);
                v
            })
            .collect();
        let mut consecs = vec![0usize; num];
        let mut run_starts = vec![0usize; num];
        let mut barriers: Vec<Vec<(usize, usize)>> = (0..num).map(|_| Vec::new()).collect();

        // Single pass: build prefix sums and detect barrier runs for all bases.
        for (i, &sym) in seq.iter().enumerate() {
            for bi in 0..num {
                let is_match = sym == check_bases[bi];
                let prev = *prefixes[bi]
                    .last()
                    .expect("Prefixes can't be empty, was filled from check_bases.len()");
                prefixes[bi].push(prev + if is_match { match_w } else { mis_w });

                if is_match {
                    if consecs[bi] >= max_consec {
                        barriers[bi].push((run_starts[bi], i - 1));
                    }
                    consecs[bi] = 0;
                } else {
                    if consecs[bi] == 0 {
                        run_starts[bi] = i;
                    }
                    consecs[bi] += 1;
                }
            }
        }
        // Finalise any barrier run that reaches the end of the sequence.
        for bi in 0..num {
            if consecs[bi] >= max_consec {
                barriers[bi].push((run_starts[bi], n - 1));
            }
        }

        let mut best: Option<(usize, usize)> = None;
        for bi in 0..num {
            for (seg_start, seg_end) in Self::barrier_free_segments(n, &barriers[bi], max_consec) {
                if seg_end + 1 - seg_start < min_length {
                    continue;
                }
                let candidate =
                    Self::longest_nonneg_subarray(&prefixes[bi], seg_start, seg_end, min_length);
                best = Self::pick_better(best, candidate);
            }
        }

        best
    }

    /// Fast path for `max_consecutive_mismatches == 0`: find the longest
    /// contiguous run of exact base matches (no mismatches at all).
    fn longest_exact_run(seq: &[u8], base: u8, min_length: usize) -> Option<(usize, usize)> {
        let check_bases: &[u8] = if base == b'.' {
            b"ACGT"
        } else {
            std::slice::from_ref(&base)
        };

        let mut best: Option<(usize, usize)> = None;
        for &b in check_bases {
            let mut run_start = 0;
            let mut in_run = false;
            for (i, &sym) in seq.iter().enumerate() {
                if sym == b {
                    if !in_run {
                        run_start = i;
                        in_run = true;
                    }
                } else if in_run {
                    let len = i - run_start;
                    if len >= min_length {
                        best = Self::pick_better(best, Some((run_start, len)));
                    }
                    in_run = false;
                }
            }
            if in_run {
                let len = seq.len() - run_start;
                if len >= min_length {
                    best = Self::pick_better(best, Some((run_start, len)));
                }
            }
        }
        best
    }

    /// Derive segments in which no run of `max_consec` consecutive mismatches
    /// occurs. Each segment may extend up to `max_consec - 1` positions into a
    /// neighbouring barrier so that regions ending/starting with a few
    /// tolerable mismatches are still discoverable.
    fn barrier_free_segments(
        n: usize,
        barriers: &[(usize, usize)],
        max_consec: usize,
    ) -> Vec<(usize, usize)> {
        if barriers.is_empty() {
            return vec![(0, n - 1)];
        }

        // How far a segment may reach into a barrier from either side.
        let ext = max_consec.saturating_sub(1);
        let mut segs = Vec::new();

        // Segment end that extends `ext` positions into a barrier starting at
        // `b_start` (i.e. b_start + ext - 1, or b_start - 1 when ext == 0).
        let end_into_barrier = |b_start: usize| -> Option<usize> {
            if ext > 0 {
                Some(b_start + ext - 1)
            } else if b_start > 0 {
                Some(b_start - 1)
            } else {
                None
            }
        };

        // Segment start that extends `ext` positions into a barrier ending at
        // `b_end` (i.e. b_end + 1 - ext, saturating to 0).
        let start_after_barrier = |b_end: usize| -> usize { (b_end + 1).saturating_sub(ext) };

        // Before first barrier.
        if let Some(se) = end_into_barrier(barriers[0].0) {
            segs.push((0, se.min(n - 1)));
        }

        // Between consecutive barriers.
        for w in barriers.windows(2) {
            let ss = start_after_barrier(w[0].1);
            if let Some(se) = end_into_barrier(w[1].0) {
                let se = se.min(n - 1);
                if ss <= se {
                    segs.push((ss, se));
                }
            } // cov:excl-line
        }

        // After last barrier.
        let ss = start_after_barrier(
            barriers
                .last()
                .expect("Barriers was checked for emptyness above")
                .1,
        );
        if ss < n {
            segs.push((ss, n - 1));
        }

        segs
    }

    /// Classic monotone-stack algorithm for the longest subarray whose
    /// prefix-sum difference is non-negative (i.e. mismatch rate <= threshold).
    /// Returns `Some((start, length))` or `None`.
    fn longest_nonneg_subarray(
        prefix: &[f64],
        seg_start: usize,
        seg_end: usize,
        min_length: usize,
    ) -> Option<(usize, usize)> {
        // Build a stack of prefix-array indices with strictly decreasing values.
        // These are the only useful candidate left-endpoints.
        let mut stack: Vec<usize> = Vec::new();
        for i in seg_start..=seg_end + 1 {
            if stack.is_empty()
                || prefix[i] < prefix[*stack.last().expect("Checked for empty just before")]
            {
                stack.push(i);
            }
        }

        let mut best: Option<(usize, usize)> = None;

        // Scan right-endpoints from large to small; each stack entry is popped
        // at most once, giving amortised O(n).
        for r in (seg_start..=seg_end + 1).rev() {
            while let Some(&l) = stack.last() {
                if prefix[l] <= prefix[r] {
                    stack.pop();
                    let length = r - l;
                    if length >= min_length {
                        best = Self::pick_better(best, Some((l, length)));
                    }
                } else {
                    break;
                }
            }
        }

        best
    }
}

impl TagUser for PartialTaggedVariant<PartialLongestPolyX> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Location),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for LongestPolyX {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self.segment;
        let min_length = self.min_length;
        let base = self.base;
        let max_mismatch_fraction = self.max_mismatch_rate;
        let max_consecutive_mismatches = self.max_consecutive_mismatches;

        extract_region_tags_from_seq(
            &mut block,
            segment_index,
            &self.out_label,
            move |read_seq| {
                Self::find_best(
                    read_seq,
                    base,
                    min_length,
                    max_mismatch_fraction,
                    max_consecutive_mismatches,
                )
                .map(|(start, len)| HitDraft {
                    location: Some(HitRegionView {
                        start,
                        len,
                        segment_index,
                    }),
                    sequence: read_seq[start..start + len].to_vec(),
                })
            },
        );
        Ok((block, true))
    }
}
