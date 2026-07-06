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
                if (new_candidate.1 == existing.1 && new_candidate.0 < existing.0) //mutants::skip
                    //(when new_candidate.0 == existing.0 as well, both branches
                    //return the identical (start, len) pair, so `<` vs `<=` here is unobservable)
                    || new_candidate.1 > existing.1
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
                barriers[bi].push((run_starts[bi], n - 1)); //mutants::skip
                // the end of this last barrier is irrelevant
            }
        }

        let mut best: Option<(usize, usize)> = None;
        for bi in 0..num {
            for (seg_start, seg_end) in Self::barrier_free_segments(n, &barriers[bi], max_consec) {
                if seg_end + 1 - seg_start < min_length {
                    //mutants::skip
                    // (optimization only: longest_nonneg_subarray already
                    // enforces min_length internally before recording any candidate, so
                    // skipping this pre-check just wastes a call on too-short segments)
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
            // mutants::skip
            if stack.is_empty()
                || prefix[i] < prefix[*stack.last().expect("Checked for empty just before")]
            //mutants::skip
            //(a duplicate pushed here has the same prefix value as the
            //entry already below it on the stack; both are only ever popped together
            //for the same `r` in the loop below, and pick_better always keeps the
            //longer (earlier-index) one, so allowing the duplicate push changes
            //nothing observable)
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
        if let Some(inner) = self.toml_value.value.as_mut()
            && let Some(Some(segment)) = inner.segment.as_ref().map(|m| m.as_ref_post())
        {
            // Record the segment this location tag lives on, so a conditional
            // `Swap` on that segment can forget it (a per-read swap would split it
            // across two segments).
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Location)
                    .map(|dt| dt.with_segment(*segment)),
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
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "lengths are guaranteed to be within u32 range"
                )]
                Self::find_best(
                    read_seq,
                    base,
                    min_length,
                    max_mismatch_fraction,
                    max_consecutive_mismatches,
                )
                .map(|(start, len)| start as u32..(start + len) as u32)
            },
        );
        Ok((block, true))
    }
}

#[cfg(test)]
mod test {
    use super::LongestPolyX;

    #[test]
    fn test_barrier_free_segments_end_into_barrier_ext_gt_zero() {
        // max_consec=3 -> ext=2, so a segment before a barrier starting at 4 may
        // extend into the barrier up to index 4+2-1=5, capped at n-1.
        assert_eq!(
            LongestPolyX::barrier_free_segments(8, &[(4, 4)], 3),
            vec![(0, 5), (3, 7)]
        );
    }

    #[test]
    fn test_barrier_free_segments_end_into_barrier_ext_zero() {
        // max_consec=1 -> ext=0, so a segment before a barrier starting at 3 may not
        // penetrate it at all: it must stop at 3-1=2.
        assert_eq!(
            LongestPolyX::barrier_free_segments(6, &[(3, 3)], 1),
            vec![(0, 2), (4, 5)]
        );
    }

    #[test]
    fn test_barrier_free_segments_caps_at_sequence_end() {
        // ext=4 is larger than the sequence itself, so both the "before first barrier"
        // and "between barriers" segment ends must be capped at n-1 (4), not left at
        // their raw (out-of-range) computed values.
        assert_eq!(
            LongestPolyX::barrier_free_segments(5, &[(4, 4)], 5),
            vec![(0, 4), (1, 4)]
        );
        assert_eq!(
            LongestPolyX::barrier_free_segments(6, &[(0, 0), (5, 5)], 6),
            vec![(0, 4), (0, 5), (1, 5)]
        );
    }

    #[test]
    fn test_barrier_free_segments_no_segment_past_sequence_end() {
        // The lone barrier touches the very last index with ext=0, so there is no room
        // for an "after last barrier" segment at all: start_after_barrier(5) == n(6),
        // which must be excluded, not merely allowed to equal n.
        assert_eq!(
            LongestPolyX::barrier_free_segments(6, &[(5, 5)], 1),
            vec![(0, 4)]
        );
    }

    #[test]
    fn test_find_best_barrier_run_start_tracking() {
        // A 3-long mismatch run (positions 10..=12) exceeds max_consecutive_mismatches
        // (2), so it becomes a barrier. The barrier must be recorded as starting at the
        // *first* mismatch of the run (position 10), not somewhere in the middle of it -
        // otherwise the "before" barrier-free segment wrongly swallows part of the
        // mismatch run and reports a longer match than actually qualifies.
        let seq = b"AAAAAAAAAATTTAAA";
        assert_eq!(LongestPolyX::find_best(seq, b'A', 3, 0.3, 2), Some((0, 11)));
    }

    #[test]
    fn test_longest_nonneg_subarray_uses_last_stack_index() {
        // A strictly decreasing run whose last element (index == seg_end) is the
        // global minimum of the whole segment, followed by a single upward step to
        // seg_end + 1. The only valid non-negative subarray is the single-element one
        // starting at seg_end: [seg_end, seg_end+1]. That requires the stack-building
        // loop to actually consider index seg_end (not stop short of it).
        let prefix = vec![0.0, -0.8, -1.6, -2.4, -2.2];
        assert_eq!(
            LongestPolyX::longest_nonneg_subarray(&prefix, 0, 3, 1),
            Some((3, 1))
        );
    }

    #[test]
    fn test_pick_better() {
        // longer candidate always wins
        assert_eq!(
            LongestPolyX::pick_better(Some((5, 3)), Some((10, 8))),
            Some((10, 8))
        );
        // shorter candidate never wins
        assert_eq!(
            LongestPolyX::pick_better(Some((5, 8)), Some((10, 3))),
            Some((5, 8))
        );
        // tie on length: earlier start wins
        assert_eq!(
            LongestPolyX::pick_better(Some((10, 5)), Some((3, 5))),
            Some((3, 5))
        );
        assert_eq!(
            LongestPolyX::pick_better(Some((3, 5)), Some((10, 5))),
            Some((3, 5))
        );
        assert_eq!(LongestPolyX::pick_better(None, Some((3, 5))), Some((3, 5)));
        assert_eq!(LongestPolyX::pick_better(Some((3, 5)), None), Some((3, 5)));
    }
}
