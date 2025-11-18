#![allow(clippy::unnecessary_wraps)]

use crate::config::{Segment, SegmentIndex};
use crate::io::WrappedFastQReadMut;
use crate::transformations::TagValue;
use crate::transformations::prelude::*;
use serde_valid::Validate;
use std::borrow::Cow;
use std::cell::RefCell;

/// Algorithm to use for scoring overlaps and resolving mismatches
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
pub enum Algorithm {
    /// fastp algorithm: quality-score based mismatch resolution
    /// Uses hamming distance for overlap detection and chooses higher quality base for mismatches
    #[serde(alias = "FastpSeemsWeird")]
    Fastp,
    /// WFA2 (Wavefront Alignment Algorithm 2): Fast gap-affine alignment
    /// More accurate than hamming distance, allows insertions/deletions in overlap
    /// Typically 10-100x faster than traditional dynamic programming for short reads
    #[serde(alias = "wfa2")]
    Wfa2,
}

/// Strategy when reads cannot be merged due to insufficient overlap
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NoOverlapStrategy {
    /// Keep reads as they are (no merging)
    AsIs,
    /// Concatenate reads with spacer into first segment
    Concatenate,
}

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MergeReads {
    /// Algorithm to use for overlap scoring and mismatch resolution
    pub algorithm: Algorithm,

    /// Minimum overlap length required for merging (suggested: 30, minimum: 5)
    #[validate(minimum = 5)]
    pub min_overlap: usize,

    /// Maximum allowed mismatch rate (0.0 to 1.0, suggested: 0.2)
    /// At least one of max_mismatch_rate or max_mismatch_count must be specified
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub max_mismatch_rate: f64,

    /// Maximum allowed absolute number of mismatches (suggested: 5)
    /// At least one of max_mismatch_rate or max_mismatch_count must be specified
    #[serde(default)]
    pub max_mismatch_count: usize,

    /// Strategy when no overlap is found (suggested: "as_is")
    pub no_overlap_strategy: NoOverlapStrategy,

    /// Tag label to store merge status (suggested: "merged")
    /// Tag will be true if reads were merged, false otherwise
    #[serde(default)]
    pub out_label: Option<String>,

    /// Spacer sequence to use when concatenating (required if no_overlap_strategy = 'concatenate')
    #[serde(default)]
    pub concatenate_spacer: Option<String>,

    /// Quality score to use for spacer bases (suggested: 33, which is Phred quality 0)
    #[validate(minimum = 33)]
    #[validate(maximum = 126)]
    #[serde(default)]
    pub spacer_quality_char: Option<u8>,

    /// Whether to reverse complement segment2 before merging
    pub reverse_complement_segment2: bool,

    pub segment1: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment1_index: Option<SegmentIndex>,

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

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        self.out_label
            .as_ref()
            .map(|label| (label.clone(), crate::transformations::TagValueType::Bool))
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
        let algorithm = self.algorithm.clone();

        // Track which reads were merged (if label is set)
        let merge_status = RefCell::new(
            self.out_label
                .as_ref()
                .map(|_| Vec::with_capacity(block.len())),
        );

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
                &algorithm,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            )
            .expect("Merge failed");

            // Apply the merge result and track if merged
            let was_merged = match merge_result {
                MergeResult::Merged {
                    merged_seq,
                    merged_qual,
                } => {
                    // Update segment1 with merged sequence
                    reads[seg1_idx].replace_seq(merged_seq, merged_qual);
                    // Clear segment2
                    reads[seg2_idx].clear();
                    true
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
                    // Otherwise keep reads as they are (NoOverlapStrategy::AsIs)
                    false
                }
            };

            // Record merge status if tracking
            if let Some(merge_status) = merge_status.borrow_mut().as_mut() {
                merge_status.push(was_merged);
            }
        });

        // Add merge status tag if label was specified

        if let Some(merge_status) = merge_status.take() {
            let tag_values: Vec<TagValue> = merge_status.into_iter().map(TagValue::Bool).collect();
            block
                .tags
                .insert(self.out_label.as_ref().unwrap().clone(), tag_values);
        }

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
    algorithm: &Algorithm,
    min_overlap: usize,
    max_mismatch_rate: f64,
    max_mismatch_count: usize,
) -> Result<MergeResult> {
    match algorithm {
        Algorithm::Fastp => {
            let ov = find_best_overlap_fastp(
                seq1,
                seq2,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            );
            if let Some((offset, overlap_len)) = ov {
                // Merge the reads
                let (merged_seq, merged_qual) =
                    merge_at_offset_fastp(seq1, qual1, seq2, qual2, offset, overlap_len)?;
                Ok(MergeResult::Merged {
                    merged_seq,
                    merged_qual,
                })
            } else {
                Ok(MergeResult::NoOverlap)
            }
        }
        Algorithm::Wfa2 => {
            let ov = find_best_overlap_wfa2(
                seq1,
                seq2,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            )?;
            if let Some((offset, overlap_len, cigar)) = ov {
                // Merge the reads using WFA2 alignment result
                let (merged_seq, merged_qual) =
                    merge_at_offset_wfa2(seq1, qual1, seq2, qual2, offset, overlap_len, &cigar)?;
                Ok(MergeResult::Merged {
                    merged_seq,
                    merged_qual,
                })
            } else {
                Ok(MergeResult::NoOverlap)
            }
        }
    }
}

/// Find the best overlap using fastp algorithm (hamming distance)
fn find_best_overlap_fastp(
    seq1: &[u8],
    seq2: &[u8],
    min_overlap: usize,
    max_mismatch_rate: f64,
    max_mismatch_count: usize,
) -> Option<(isize, usize)> {
    let len1 = seq1.len();
    let len2 = seq2.len();

    let mut best_match: Option<(isize, usize, usize)> = None; // (offset, overlap_len, mismatches)

    // Phase 1: Forward alignment (seq2 starts inside seq1)
    // offset is the position in seq1 where seq2 starts
    let max_offset = len1.saturating_sub(min_overlap + 1);
    for offset in 0..=max_offset {
        let overlap_len = (len1 - offset).min(len2);
        if overlap_len < min_overlap {
            break;
        }

        let mismatches = bio::alignment::distance::hamming(
            &seq1[offset..offset + overlap_len],
            &seq2[..overlap_len],
        ) as usize;

        let first_k_below_limit = {
            if overlap_len < 50 {
                false
            } else {
                bio::alignment::distance::hamming(&seq1[offset..offset + 50], &seq2[..50]) as usize
                    <= max_mismatch_count
            }
        };

        let max_mismatches =
            max_mismatch_count.min((overlap_len as f64 * max_mismatch_rate) as usize);
        if mismatches <= max_mismatches || (first_k_below_limit) {
            if best_match.is_none() || mismatches < best_match.unwrap().2 {
                best_match = Some((offset as isize, overlap_len, mismatches));
            }
        }
    }
    if best_match.is_none() {
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

            let max_mismatches =
                max_mismatch_count.min((overlap_len as f64 * max_mismatch_rate) as usize);

            if mismatches <= max_mismatches {
                let neg_offset = -(offset as isize);
                if best_match.is_none() || mismatches < best_match.unwrap().2 {
                    best_match = Some((neg_offset, overlap_len, mismatches));
                }
            }
        }
    }

    best_match.map(|(offset, overlap_len, _)| (offset, overlap_len))
}

/// fastp is documented to prefer R1 bases, no matter what.
fn merge_at_offset_fastp(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    offset: isize,
    overlap_len: usize,
) -> Result<(Vec<u8>, Vec<u8>)> {
    fn append_overlap(
        seq1: &[u8],
        qual1: &[u8],
        seq2: &[u8],
        qual2: &[u8],
        overlap_len: usize,
        merged_seq: &mut Vec<u8>,
        merged_qual: &mut Vec<u8>,
        offset: usize,
        inside_out: bool,
    ) {
        for i in 0..overlap_len {
            let (pos1, pos2) = if inside_out {
                (i, offset + i)
            } else {
                (offset + i, i)
            };

            let base1 = seq1[pos1];
            let base2 = seq2[pos2];
            let q1 = qual1[pos1];
            let q2 = qual2[pos2];

            if base1 == base2 {
                // Agreement - use base1.
                merged_seq.push(base1);
                merged_qual.push(q1);
            } else {
                // Mismatch - fastp does something special here..
                const GOOD_QUAL: u8 = 30 + 33;
                const BAD_QUAL: u8 = 14 + 33;
                if q1 >= GOOD_QUAL && q2 <= BAD_QUAL {
                    // use R1
                    merged_seq.push(base1);
                    merged_qual.push(q1);
                } else if q2 >= GOOD_QUAL && q1 <= BAD_QUAL {
                    // use R2
                    merged_seq.push(base2);
                    merged_qual.push(q2);
                } else {
                    //use r1
                    merged_seq.push(base1);
                    merged_qual.push(q1);
                }
            }
        }
    }
    let mut merged_seq = Vec::new();
    let mut merged_qual = Vec::new();

    if offset >= 0 {
        let offset = offset as usize;
        // Non-overlapping part of seq1
        merged_seq.extend_from_slice(&seq1[..offset]);
        merged_qual.extend_from_slice(&qual1[..offset]);

        // Overlapping part - resolve mismatches using quality scores
        append_overlap(
            seq1,
            qual1,
            seq2,
            qual2,
            overlap_len,
            &mut merged_seq,
            &mut merged_qual,
            offset,
            false,
        );

        // Non-overlapping part of seq2
        if overlap_len < seq2.len() {
            merged_seq.extend_from_slice(&seq2[overlap_len..]);
            merged_qual.extend_from_slice(&qual2[overlap_len..]);
        }
    } else {
        let offset = (-offset) as usize;
        // fastp  only keeps the overlapping part.
        // Overlapping part - resolve mismatches using quality scores
        append_overlap(
            seq1,
            qual1,
            seq2,
            qual2,
            overlap_len,
            &mut merged_seq,
            &mut merged_qual,
            offset,
            true,
        );
    }

    Ok((merged_seq, merged_qual))
}

/// Find the best overlap using WFA2 algorithm (gap-affine alignment)
/// Returns (offset, overlap_len, cigar) if overlap found
fn find_best_overlap_wfa2(
    seq1: &[u8],
    seq2: &[u8],
    min_overlap: usize,
    max_mismatch_rate: f64,
    max_mismatch_count: usize,
) -> Result<Option<(isize, usize, Vec<u8>)>> {
    use libwfa::affine_wavefront::*;
    use libwfa::mm_allocator::*;
    use libwfa::penalties::AffinePenalties;

    let len1 = seq1.len();
    let len2 = seq2.len();

    const BUFFER_SIZE_8M: usize = 8 * 1024 * 1024;
    let alloc = MMAllocator::new(BUFFER_SIZE_8M as u64);

    let mut penalties = AffinePenalties {
        match_: 0,
        mismatch: 4,
        gap_opening: 6,
        gap_extension: 2,
    };

    let mut best_match: Option<(isize, usize, Vec<u8>)> = None;

    // Phase 1: Forward alignment (seq2 starts inside seq1)
    // Try different offsets where seq2 starts in seq1
    let max_offset = len1.saturating_sub(min_overlap);
    for offset in 0..=max_offset {
        let overlap_len = (len1 - offset).min(len2);
        if overlap_len < min_overlap {
            break;
        }

        // Align the overlapping regions
        let pattern = &seq1[offset..offset + overlap_len];
        let text = &seq2[..overlap_len];

        let mut wavefronts = AffineWavefronts::new_complete(
            pattern.len(),
            text.len(),
            &mut penalties,
            &alloc,
        );

        if wavefronts.align(pattern, text).is_ok() {
            let score = wavefronts.edit_cigar_score(&mut penalties);
            let cigar = wavefronts.cigar_bytes_raw();

            // Calculate number of mismatches/indels from score
            let edits = (score / 2) as usize; // Rough estimate: mismatch=4, so score/2 ≈ edits
            let max_edits = max_mismatch_count.min((overlap_len as f64 * max_mismatch_rate) as usize);

            if edits <= max_edits {
                if best_match.is_none() || edits < best_match.as_ref().unwrap().1 {
                    best_match = Some((offset as isize, overlap_len, cigar));
                }
                break; // Found good match, prefer earlier offset
            }
        }
    }

    // Phase 2: Reverse alignment (seq1 starts inside seq2)
    if best_match.is_none() {
        let max_offset = len2.saturating_sub(min_overlap);
        for offset in 1..=max_offset {
            let overlap_len = (len2 - offset).min(len1);
            if overlap_len < min_overlap {
                break;
            }

            // Align the overlapping regions
            let pattern = &seq2[offset..offset + overlap_len];
            let text = &seq1[..overlap_len];

            let mut wavefronts = AffineWavefronts::new_complete(
                pattern.len(),
                text.len(),
                &mut penalties,
                &alloc,
            );

            if wavefronts.align(pattern, text).is_ok() {
                let score = wavefronts.edit_cigar_score(&mut penalties);
                let cigar = wavefronts.cigar_bytes_raw();

                let edits = (score / 2) as usize;
                let max_edits = max_mismatch_count.min((overlap_len as f64 * max_mismatch_rate) as usize);

                if edits <= max_edits {
                    best_match = Some((-(offset as isize), overlap_len, cigar));
                    break;
                }
            }
        }
    }

    Ok(best_match)
}

/// Merge two sequences using WFA2 alignment (handles indels via CIGAR)
fn merge_at_offset_wfa2(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    offset: isize,
    _overlap_len: usize,
    cigar: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut merged_seq = Vec::new();
    let mut merged_qual = Vec::new();

    if offset >= 0 {
        let offset = offset as usize;

        // Add non-overlapping prefix from seq1
        merged_seq.extend_from_slice(&seq1[..offset]);
        merged_qual.extend_from_slice(&qual1[..offset]);

        // Process the overlapping region using CIGAR
        let (overlap_seq, overlap_qual) = merge_with_cigar(
            &seq1[offset..],
            &qual1[offset..],
            seq2,
            qual2,
            cigar,
        )?;

        merged_seq.extend_from_slice(&overlap_seq);
        merged_qual.extend_from_slice(&overlap_qual);
    } else {
        // Negative offset: seq1 starts inside seq2
        // For negative offsets, we use the overlapping alignment result
        let (overlap_seq, overlap_qual) = merge_with_cigar(
            seq1,
            qual1,
            seq2,
            qual2,
            cigar,
        )?;

        merged_seq.extend_from_slice(&overlap_seq);
        merged_qual.extend_from_slice(&overlap_qual);
    }

    Ok((merged_seq, merged_qual))
}

/// Merge two sequences using CIGAR string guidance
/// CIGAR operations: M (match/mismatch), I (insertion to ref), D (deletion from ref), X (mismatch), = (match)
fn merge_with_cigar(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    cigar: &[u8],
) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut merged_seq = Vec::new();
    let mut merged_qual = Vec::new();

    let mut pos1 = 0;
    let mut pos2 = 0;

    // Parse CIGAR string
    let mut i = 0;
    while i < cigar.len() {
        // Read the count
        let mut count = 0usize;
        while i < cigar.len() && cigar[i].is_ascii_digit() {
            count = count * 10 + (cigar[i] - b'0') as usize;
            i += 1;
        }

        if i >= cigar.len() {
            break;
        }

        // Read the operation
        let op = cigar[i];
        i += 1;

        match op {
            b'M' | b'=' | b'X' => {
                // Match or mismatch - take from both sequences, prefer higher quality
                for _ in 0..count {
                    if pos1 < seq1.len() && pos2 < seq2.len() {
                        let q1 = qual1[pos1];
                        let q2 = qual2[pos2];

                        if q1 >= q2 {
                            merged_seq.push(seq1[pos1]);
                            merged_qual.push(q1);
                        } else {
                            merged_seq.push(seq2[pos2]);
                            merged_qual.push(q2);
                        }
                        pos1 += 1;
                        pos2 += 1;
                    }
                }
            }
            b'I' => {
                // Insertion in seq2 (relative to seq1) - take from seq2
                for _ in 0..count {
                    if pos2 < seq2.len() {
                        merged_seq.push(seq2[pos2]);
                        merged_qual.push(qual2[pos2]);
                        pos2 += 1;
                    }
                }
            }
            b'D' => {
                // Deletion in seq2 (present in seq1) - take from seq1
                for _ in 0..count {
                    if pos1 < seq1.len() {
                        merged_seq.push(seq1[pos1]);
                        merged_qual.push(qual1[pos1]);
                        pos1 += 1;
                    }
                }
            }
            _ => {
                // Unknown operation, skip
            }
        }
    }

    // Append any remaining sequence from seq2
    if pos2 < seq2.len() {
        merged_seq.extend_from_slice(&seq2[pos2..]);
        merged_qual.extend_from_slice(&qual2[pos2..]);
    }

    Ok((merged_seq, merged_qual))
}

/*
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
} */
