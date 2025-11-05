#![allow(clippy::unnecessary_wraps)]

use crate::config::{Segment, SegmentIndex};
use crate::io::WrappedFastQReadMut;
use crate::transformations::prelude::*;
use serde_valid::Validate;
use std::borrow::Cow;

/// Strategy when reads cannot be merged due to insufficient overlap
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoOverlapStrategy {
    /// Keep reads as they are (no merging)
    Keep,
    /// Concatenate reads with spacer into first segment
    Concatenate,
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct MergeReads {
    /// Minimum overlap length required for merging (suggested: 30, minimum: 5)
    #[validate(minimum = 5)]
    pub min_overlap: usize,

    /// Maximum allowed mismatch rate (0.0 to 1.0, suggested: 0.2)
    /// At least one of max_mismatch_rate or max_mismatch_count must be specified
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    #[serde(default)]
    pub max_mismatch_rate: Option<f64>,

    /// Maximum allowed absolute number of mismatches (suggested: 5)
    /// At least one of max_mismatch_rate or max_mismatch_count must be specified
    #[serde(default)]
    pub max_mismatch_count: Option<usize>,

    /// Allow single gap (insertion/deletion) during alignment (suggested: false)
    pub allow_gap: bool,

    /// Strategy when no overlap is found (suggested: "keep")
    pub no_overlap_strategy: NoOverlapStrategy,

    /// Spacer sequence to use when concatenating (required if no_overlap_strategy = 'concatenate')
    #[serde(default)]
    pub concatenate_spacer: Option<String>,

    /// Quality score to use for spacer bases (suggested: 33, which is Phred quality 0)
    #[validate(minimum = 33)]
    #[validate(maximum = 126)]
    #[serde(default)]
    pub spacer_quality_char: Option<u8>,

    /// Whether to reverse complement segment2 before merging (suggested: true for paired-end reads)
    pub reverse_complement_segment2: bool,

    /// First segment (typically read1, suggested: "read1")
    pub segment1: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment1_index: Option<SegmentIndex>,

    /// Second segment (typically read2, suggested: "read2")
    pub segment2: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment2_index: Option<SegmentIndex>,
}

impl Step for MergeReads {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment1_index = Some(self.segment1.validate(input_def)?);
        self.segment2_index = Some(self.segment2.validate(input_def)?);

        // Ensure they're different segments
        if self.segment1_index == self.segment2_index {
            bail!("segment1 and segment2 must be different segments");
        }

        // Validate concatenate_spacer requirement
        if self.no_overlap_strategy == NoOverlapStrategy::Concatenate
            && self.concatenate_spacer.is_none()
        {
            bail!("concatenate_spacer is required when no_overlap_strategy = 'concatenate'");
        }

        // Validate that at least one mismatch parameter is specified
        if self.max_mismatch_rate.is_none() && self.max_mismatch_count.is_none() {
            bail!("At least one of max_mismatch_rate or max_mismatch_count must be specified");
        }

        Ok(())
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let seg1_idx = self.segment1_index.unwrap().get_index();
        let seg2_idx = self.segment2_index.unwrap().get_index();
        let reverse_complement = self.reverse_complement_segment2;
        let no_overlap_strategy = self.no_overlap_strategy.clone();
        let concatenate_spacer = self.concatenate_spacer.clone();
        let spacer_qual = self.spacer_quality_char.unwrap_or(33);
        let min_overlap = self.min_overlap;
        let max_mismatch_rate = self.max_mismatch_rate;
        let max_mismatch_count = self.max_mismatch_count;

        // Process each read pair using apply_mut
        block.apply_mut(|reads: &mut [WrappedFastQReadMut]| {
            let read1_seq = reads[seg1_idx].seq();
            let read1_qual = reads[seg1_idx].qual();
            let read2_seq = reads[seg2_idx].seq();
            let read2_qual = reads[seg2_idx].qual();

            // Optionally reverse complement read2
            let (read2_seq_processed, read2_qual_processed): (Cow<[u8]>, Cow<[u8]>) =
                if reverse_complement {
                    let rc_seq = crate::dna::reverse_complement(read2_seq);
                    let rc_qual: Vec<u8> = read2_qual.iter().rev().copied().collect();
                    (Cow::Owned(rc_seq), Cow::Owned(rc_qual))
                } else {
                    (Cow::Borrowed(read2_seq), Cow::Borrowed(read2_qual))
                };

            // Try to find overlap and merge
            let merge_result = try_merge_reads(
                read1_seq,
                read1_qual,
                &read2_seq_processed,
                &read2_qual_processed,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            )
            .expect("Merge failed");

            // Apply the merge result
            match merge_result {
                MergeResult::Merged {
                    merged_seq,
                    merged_qual,
                } => {
                    // Update segment1 with merged sequence
                    reads[seg1_idx].replace_seq(merged_seq, merged_qual);
                    // Clear segment2
                    reads[seg2_idx].clear();
                }
                MergeResult::NoOverlap => {
                    // Handle according to strategy
                    if no_overlap_strategy == NoOverlapStrategy::Concatenate {
                        let spacer = concatenate_spacer.as_ref().unwrap();

                        // Concatenate read1 + spacer + read2_processed into segment1
                        let mut concatenated_seq = read1_seq.to_vec();
                        concatenated_seq.extend_from_slice(spacer.as_bytes());
                        concatenated_seq.extend_from_slice(&read2_seq_processed);

                        let mut concatenated_qual = read1_qual.to_vec();
                        concatenated_qual.extend(std::iter::repeat(spacer_qual).take(spacer.len()));
                        concatenated_qual.extend_from_slice(&read2_qual_processed);

                        // Update segment1 with concatenated sequence
                        reads[seg1_idx].replace_seq(concatenated_seq, concatenated_qual);
                        // Clear segment2
                        reads[seg2_idx].clear();
                    }
                    // Otherwise keep reads as they are (NoOverlapStrategy::Keep)
                }
            }
        });

        Ok((block, true))
    }
}

#[derive(Debug)]
enum MergeResult {
    Merged {
        merged_seq: Vec<u8>,
        merged_qual: Vec<u8>,
    },
    NoOverlap,
}

/// Try to merge two reads by finding their overlap
/// seq2 should already be processed (reverse complemented if needed)
fn try_merge_reads(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    min_overlap: usize,
    max_mismatch_rate: Option<f64>,
    max_mismatch_count: Option<usize>,
) -> Result<MergeResult> {
    // Try to find the best overlap
    let overlap = find_best_overlap(seq1, seq2, min_overlap, max_mismatch_rate, max_mismatch_count);

    if let Some((offset, overlap_len)) = overlap {
        // Merge the reads
        let (merged_seq, merged_qual) =
            merge_at_offset(seq1, qual1, seq2, qual2, offset, overlap_len)?;
        Ok(MergeResult::Merged {
            merged_seq,
            merged_qual,
        })
    } else {
        Ok(MergeResult::NoOverlap)
    }
}

/// Find the best overlap between two sequences
/// Returns (offset, overlap_length) if a valid overlap is found
/// If both max_mismatch_rate and max_mismatch_count are specified, both conditions must be met
fn find_best_overlap(
    seq1: &[u8],
    seq2: &[u8],
    min_overlap: usize,
    max_mismatch_rate: Option<f64>,
    max_mismatch_count: Option<usize>,
) -> Option<(isize, usize)> {
    let len1 = seq1.len();
    let len2 = seq2.len();

    let mut best_match: Option<(isize, usize, usize)> = None; // (offset, overlap_len, mismatches)

    // Phase 1: Forward alignment (seq2 starts inside seq1)
    // offset is the position in seq1 where seq2 starts
    let max_offset = len1.saturating_sub(min_overlap);
    for offset in 0..=max_offset {
        let overlap_len = (len1 - offset).min(len2);
        if overlap_len < min_overlap {
            break;
        }

        let mismatches = bio::alignment::distance::hamming(
            &seq1[offset..offset + overlap_len],
            &seq2[..overlap_len],
        ) as usize;

        // Check both conditions if specified
        let mut passes = true;

        if let Some(max_rate) = max_mismatch_rate {
            let mismatch_rate = mismatches as f64 / overlap_len as f64;
            if mismatch_rate > max_rate {
                passes = false;
            }
        }

        if let Some(max_count) = max_mismatch_count {
            if mismatches > max_count {
                passes = false;
            }
        }

        if passes {
            if best_match.is_none() || mismatches < best_match.unwrap().2 {
                best_match = Some((offset as isize, overlap_len, mismatches));
            }
        }
    }

    // Phase 2: Reverse alignment (seq1 starts inside seq2)
    // negative offset means seq2 starts before seq1
    let max_offset = len2.saturating_sub(min_overlap);
    for offset in 1..=max_offset {
        let overlap_len = (len2 - offset).min(len1);
        if overlap_len < min_overlap {
            break;
        }

        let mismatches = bio::alignment::distance::hamming(
            &seq2[offset..offset + overlap_len],
            &seq1[..overlap_len],
        ) as usize;

        // Check both conditions if specified
        let mut passes = true;

        if let Some(max_rate) = max_mismatch_rate {
            let mismatch_rate = mismatches as f64 / overlap_len as f64;
            if mismatch_rate > max_rate {
                passes = false;
            }
        }

        if let Some(max_count) = max_mismatch_count {
            if mismatches > max_count {
                passes = false;
            }
        }

        if passes {
            let neg_offset = -(offset as isize);
            if best_match.is_none() || mismatches < best_match.unwrap().2 {
                best_match = Some((neg_offset, overlap_len, mismatches));
            }
        }
    }

    best_match.map(|(offset, overlap_len, _)| (offset, overlap_len))
}

/// Merge two sequences at the given offset
fn merge_at_offset(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    offset: isize,
    overlap_len: usize,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut merged_seq = Vec::new();
    let mut merged_qual = Vec::new();

    if offset >= 0 {
        let offset = offset as usize;
        // seq2 starts at position 'offset' in seq1
        // merged = seq1[0..offset] + merge(overlap) + seq2[overlap_len..]

        // Non-overlapping part of seq1
        merged_seq.extend_from_slice(&seq1[..offset]);
        merged_qual.extend_from_slice(&qual1[..offset]);

        // Overlapping part - resolve mismatches using quality scores
        for i in 0..overlap_len {
            let pos1 = offset + i;
            let pos2 = i;

            let base1 = seq1[pos1];
            let base2 = seq2[pos2];
            let q1 = qual1[pos1];
            let q2 = qual2[pos2];

            if base1 == base2 {
                // Agreement - use the base with higher quality
                merged_seq.push(base1);
                merged_qual.push(q1.max(q2));
            } else {
                // Mismatch - use the base with higher quality
                if q1 >= q2 {
                    merged_seq.push(base1);
                    merged_qual.push(q1);
                } else {
                    merged_seq.push(base2);
                    merged_qual.push(q2);
                }
            }
        }

        // Non-overlapping part of seq2
        if overlap_len < seq2.len() {
            merged_seq.extend_from_slice(&seq2[overlap_len..]);
            merged_qual.extend_from_slice(&qual2[overlap_len..]);
        }
    } else {
        let offset = (-offset) as usize;
        // seq1 starts at position 'offset' in seq2
        // merged = seq2[0..offset] + merge(overlap) + seq1[overlap_len..]

        // Non-overlapping part of seq2
        merged_seq.extend_from_slice(&seq2[..offset]);
        merged_qual.extend_from_slice(&qual2[..offset]);

        // Overlapping part - resolve mismatches using quality scores
        for i in 0..overlap_len {
            let pos1 = i;
            let pos2 = offset + i;

            let base1 = seq1[pos1];
            let base2 = seq2[pos2];
            let q1 = qual1[pos1];
            let q2 = qual2[pos2];

            if base1 == base2 {
                // Agreement - use the base with higher quality
                merged_seq.push(base1);
                merged_qual.push(q1.max(q2));
            } else {
                // Mismatch - use the base with higher quality
                if q1 >= q2 {
                    merged_seq.push(base1);
                    merged_qual.push(q1);
                } else {
                    merged_seq.push(base2);
                    merged_qual.push(q2);
                }
            }
        }

        // Non-overlapping part of seq1
        if overlap_len < seq1.len() {
            merged_seq.extend_from_slice(&seq1[overlap_len..]);
            merged_qual.extend_from_slice(&qual1[overlap_len..]);
        }
    }

    Ok((merged_seq, merged_qual))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_overlap_perfect_match() {
        // seq1:     ATCGATCGATCG
        // seq2:       ATCGATCGNNNN
        // overlap at offset 4, length 8
        let seq1 = b"ATCGATCGATCG";
        let seq2 = b"ATCGATCGNNNN";

        let result = find_best_overlap(seq1, seq2, 5, Some(0.0), None);
        assert!(result.is_some());
        let (offset, overlap_len) = result.unwrap();
        assert_eq!(offset, 4);
        assert_eq!(overlap_len, 8);
    }

    #[test]
    fn test_find_overlap_with_count_only() {
        let seq1 = b"ATCGATCGATCG";
        let seq2 = b"ATCGATCGNNNN";

        // Allow up to 2 mismatches (the 2 N's)
        let result = find_best_overlap(seq1, seq2, 5, None, Some(2));
        assert!(result.is_some());
        let (offset, overlap_len) = result.unwrap();
        assert_eq!(offset, 4);
        assert_eq!(overlap_len, 8);
    }

    #[test]
    fn test_find_overlap_with_both_limits() {
        let seq1 = b"ATCGATCGATCG";
        let seq2 = b"NTCGNTCGAAAA";  // 2 mismatches in overlap region (N's at positions 0 and 4)

        // Both conditions must be met: rate <= 0.3 (25% = 2/8) AND count <= 3
        let result = find_best_overlap(seq1, seq2, 5, Some(0.3), Some(3));
        assert!(result.is_some());

        // But if count is too strict, should fail (2 mismatches > 1)
        let result = find_best_overlap(seq1, seq2, 5, Some(0.3), Some(1));
        assert!(result.is_none());
    }
}
