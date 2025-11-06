#![allow(clippy::unnecessary_wraps)]

use crate::config::{Segment, SegmentIndex};
use crate::io::WrappedFastQReadMut;
use crate::transformations::TagValue;
use crate::transformations::prelude::*;
use serde_valid::Validate;
use std::borrow::Cow;
use std::cell::RefCell;

/// Algorithm to use for scoring overlaps and resolving mismatches
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Algorithm {
    /// fastp algorithm: quality-score based mismatch resolution
    /// Uses hamming distance for overlap detection and chooses higher quality base for mismatches
    #[serde(alias = "fastp_seems_weird")]
    Fastp,
    /// Simple Bayesian algorithm from pandaseq
    /// Uses Bayesian probability with pmatch/pmismatch parameters (default q=0.36)
    SimpleBayes,
    /// RDP MLE (Maximum Likelihood Estimation) algorithm from pandaseq
    /// Adjusts for MiSeq error patterns, uses higher quality score for matches
    RdpMle,
}

/// Strategy when reads cannot be merged due to insufficient overlap
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoOverlapStrategy {
    /// Keep reads as they are (no merging)
    AsIs,
    /// Concatenate reads with spacer into first segment
    Concatenate,
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct MergeReads {
    /// Algorithm to use for overlap scoring and mismatch resolution
    /// Options: "fastp", "simple_bayes", "rdp_mle"
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
    pub label: Option<String>,

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
        self.label
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
        let merge_status =
            RefCell::new(self.label.as_ref().map(|_| Vec::with_capacity(block.len())));

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

        if block.tags.is_none() {
            block.tags = Some(std::collections::HashMap::new());
        }
        if let Some(merge_status) = merge_status.take() {
            let tag_values: Vec<TagValue> = merge_status.into_iter().map(TagValue::Bool).collect();
            block
                .tags
                .as_mut()
                .unwrap()
                .insert(self.label.as_ref().unwrap().clone(), tag_values);
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
        Algorithm::SimpleBayes => {
            let ov = find_best_overlap_simple_bayes(
                seq1,
                qual1,
                seq2,
                qual2,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            );
            todo!()
        }
        Algorithm::RdpMle => {
            let ov = find_best_overlap_rdp_mle(
                seq1,
                qual1,
                seq2,
                qual2,
                min_overlap,
                max_mismatch_rate,
                max_mismatch_count,
            );
            todo!();
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

/// Find the best overlap using simple Bayesian probability (pandaseq algorithm)
fn find_best_overlap_simple_bayes(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    min_overlap: usize,
    _max_mismatch_rate: f64,
    _max_mismatch_count: usize,
) -> Option<(isize, usize)> {
    // Default error probability from pandaseq (q = 0.36)
    let q = 0.36_f64;
    let pmatch = (0.25 * (1.0 - 2.0 * q + q * q)).ln();
    let pmismatch = ((3.0 * q - 2.0 * q * q) / 18.0).ln();

    let len1 = seq1.len();
    let len2 = seq2.len();

    let mut best_match: Option<(isize, usize, f64)> = None; // (offset, overlap_len, log_probability)

    // Phase 1: Forward alignment (seq2 starts inside seq1)
    let max_offset = len1.saturating_sub(min_overlap);
    for offset in 0..=max_offset {
        let overlap_len = (len1 - offset).min(len2);
        if overlap_len < min_overlap {
            break;
        }

        let score = calculate_overlap_score_simple_bayes(
            &seq1[offset..offset + overlap_len],
            &qual1[offset..offset + overlap_len],
            &seq2[..overlap_len],
            &qual2[..overlap_len],
            pmatch,
            pmismatch,
        );

        if best_match.is_none() || score > best_match.unwrap().2 {
            best_match = Some((offset as isize, overlap_len, score));
        }
    }

    // Phase 2: Reverse alignment (seq1 starts inside seq2)
    let max_offset = len2.saturating_sub(min_overlap);
    for offset in 1..=max_offset {
        let overlap_len = (len2 - offset).min(len1);
        if overlap_len < min_overlap {
            break;
        }

        let score = calculate_overlap_score_simple_bayes(
            &seq1[..overlap_len],
            &qual1[..overlap_len],
            &seq2[offset..offset + overlap_len],
            &qual2[offset..offset + overlap_len],
            pmatch,
            pmismatch,
        );

        if best_match.is_none() || score > best_match.unwrap().2 {
            let neg_offset = -(offset as isize);
            best_match = Some((neg_offset, overlap_len, score));
        }
    }

    // For now, accept any overlap with positive log probability
    best_match
        .filter(|(_, _, score)| *score > f64::NEG_INFINITY)
        .map(|(offset, overlap_len, _)| (offset, overlap_len))
}

/// Calculate the Bayesian overlap score
fn calculate_overlap_score_simple_bayes(
    seq1: &[u8],
    _qual1: &[u8],
    seq2: &[u8],
    _qual2: &[u8],
    pmatch: f64,
    pmismatch: f64,
) -> f64 {
    let mut score = 0.0;

    for i in 0..seq1.len() {
        let b1 = seq1[i];
        let b2 = seq2[i];

        // Check if bases match (accounting for IUPAC codes if needed)
        if b1 == b2 && b1 != b'N' {
            score += pmatch;
        } else if b1 != b'N' && b2 != b'N' {
            score += pmismatch;
        }
        // Unknown bases (N) don't contribute to score
    }

    score
}

/// Find the best overlap using RDP MLE algorithm (pandaseq)
fn find_best_overlap_rdp_mle(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    min_overlap: usize,
    _max_mismatch_rate: f64,
    _max_mismatch_count: usize,
) -> Option<(isize, usize)> {
    let len1 = seq1.len();
    let len2 = seq2.len();

    let mut best_match: Option<(isize, usize, f64)> = None; // (offset, overlap_len, log_probability)

    // Phase 1: Forward alignment (seq2 starts inside seq1)
    let max_offset = len1.saturating_sub(min_overlap);
    for offset in 0..=max_offset {
        let overlap_len = (len1 - offset).min(len2);
        if overlap_len < min_overlap {
            break;
        }

        let score = calculate_overlap_score_rdp_mle(
            &seq1[offset..offset + overlap_len],
            &qual1[offset..offset + overlap_len],
            &seq2[..overlap_len],
            &qual2[..overlap_len],
        );

        if best_match.is_none() || score > best_match.unwrap().2 {
            best_match = Some((offset as isize, overlap_len, score));
        }
    }

    // Phase 2: Reverse alignment (seq1 starts inside seq2)
    let max_offset = len2.saturating_sub(min_overlap);
    for offset in 1..=max_offset {
        let overlap_len = (len2 - offset).min(len1);
        if overlap_len < min_overlap {
            break;
        }

        let score = calculate_overlap_score_rdp_mle(
            &seq1[..overlap_len],
            &qual1[..overlap_len],
            &seq2[offset..offset + overlap_len],
            &qual2[offset..offset + overlap_len],
        );

        if best_match.is_none() || score > best_match.unwrap().2 {
            let neg_offset = -(offset as isize);
            best_match = Some((neg_offset, overlap_len, score));
        }
    }

    // For now, accept any overlap with positive log probability
    best_match
        .filter(|(_, _, score)| *score > f64::NEG_INFINITY)
        .map(|(offset, overlap_len, _)| (offset, overlap_len))
}

/// Calculate the RDP MLE overlap score
/// Uses quality scores and takes the higher quality score for matching bases
fn calculate_overlap_score_rdp_mle(seq1: &[u8], qual1: &[u8], seq2: &[u8], qual2: &[u8]) -> f64 {
    let mut score = 0.0;

    for i in 0..seq1.len() {
        let b1 = seq1[i];
        let b2 = seq2[i];
        let q1 = qual1[i];
        let q2 = qual2[i];

        // Convert PHRED scores to probabilities
        let p1 = phred_to_prob(q1);
        let p2 = phred_to_prob(q2);

        // Check if bases match
        if b1 == b2 && b1 != b'N' {
            // For matching bases, use higher quality score (lower error probability)
            let match_prob = 1.0 - p1.min(p2);
            score += match_prob.ln();
        } else if b1 != b'N' && b2 != b'N' {
            // For mismatches, penalize based on both quality scores
            let mismatch_prob = (p1 * p2) / 3.0; // Divide by 3 for 3 alternative bases
            score += mismatch_prob.ln();
        }
        // Unknown bases (N) don't contribute to score
    }

    score
}

/// Convert PHRED quality score to error probability
#[inline]
fn phred_to_prob(qual: u8) -> f64 {
    let q = qual.saturating_sub(33) as f64; // Convert ASCII to PHRED
    10.0_f64.powf(-q / 10.0)
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
                // Mismatch - fastp does something special ehere..
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
        // seq2 starts at position 'offset' in seq1
        // merged = seq1[0..offset] + merge(overlap) + seq2[overlap_len..]

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
