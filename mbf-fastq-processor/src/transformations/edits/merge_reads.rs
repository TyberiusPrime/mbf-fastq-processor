#![allow(clippy::unnecessary_wraps)]

use crate::io::WrappedFastQReadMut;
use crate::transformations::TagValue;
use crate::transformations::prelude::*;
use std::borrow::Cow;
use std::cell::RefCell;

/// Algorithm to use for scoring overlaps and resolving mismatches
#[derive(Clone, PartialEq, Eq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum Algorithm {
    /// fastp algorithm: quality-score based mismatch resolution
    /// Uses hamming distance for overlap detection and chooses higher quality base for mismatches
    #[tpd(alias="FastpSeemsWeird")]
    Fastp,
}

/// Strategy when reads cannot be merged due to insufficient overlap
#[derive(Clone, PartialEq, Eq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum NoOverlapStrategy {
    /// Keep reads as they are (no merging)
    AsIs,
    /// Concatenate reads with spacer into first segment
    Concatenate,
}

/// Merge paired end reads if they're overlapping
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct MergeReads {
    /// Algorithm to use for overlap scoring and mismatch resolution
    pub algorithm: Algorithm,

    /// Minimum overlap length required for merging (suggested: 30, minimum: 5)
    pub min_overlap: usize,

    /// Maximum allowed mismatch rate (0.0 to 1.0, suggested: 0.2)
    pub max_mismatch_rate: f64,

    /// Maximum allowed absolute number of mismatches (suggested: 5)
    pub max_mismatch_count: usize,

    /// Strategy when no overlap is found
    pub no_overlap_strategy: NoOverlapStrategy,

    /// Tag label to store merge status (suggested: "merged")
    /// Tag will be true if reads were merged, false otherwise
    pub out_label: Option<String>,

    /// Spacer sequence to use when concatenating (required if `no_overlap_strategy` = 'concatenate')
    pub concatenate_spacer: Option<String>,

    /// Quality score to use for spacer bases (suggested: 33, which is Phred quality 0)
    ///#[validate(minimum = 33)] TODO
    ///#[validate(maximum = 126)]
    pub spacer_quality_char: Option<u8>,

    /// Whether to reverse complement segment2 before merging
    pub reverse_complement_segment2: bool,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment1: SegmentIndex,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment2: SegmentIndex,
}

impl VerifyIn<PartialConfig> for PartialMergeReads {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment1.validate_segment(parent);
        self.segment2.validate_segment(parent);
        if let Some(segment1) = self.segment1.as_ref()
            && let Some(segment2) = self.segment2.as_ref()
            && let MustAdapt::PostVerify(index1) = segment1
            && let MustAdapt::PostVerify(index2) = segment2
            && index1 == index2
        {
            let spans = vec![
                (
                    self.segment1.span(),
                    "Must be different from segment2".to_string(),
                ),
                (
                    self.segment2.span(),
                    "Must be different from segment2".to_string(),
                ),
            ];
            self.segment1 = TomlValue::new_custom(
                Some(MustAdapt::PostVerify(*index1)),
                spans,
                Some(&format!(
                    "Available segments: {}",
                    parent
                        .input
                        .as_ref()
                        .expect("Expected input_def to be present at this point")
                        .get_segment_order()
                        .join(", ")
                )),
            );
        }

        if let Some(no_overlap_strategy) = self.no_overlap_strategy.as_ref()
            && let Some(concatenate_spacer) = self.concatenate_spacer.as_ref()
            && *no_overlap_strategy == NoOverlapStrategy::Concatenate
            && concatenate_spacer.is_none()
        {
            return Err(ValidationFailure::new(
                "Missing key: 'concatenate_spacer'",
                Some(
                    "concatenate_spacer is required when no_overlap_strategy = 'concatenate'. Please specify a spacer sequence (e.g., concatenate_spacer = \"NNNN\").",
                ),
            ));
        }

        Ok(())
    }
}

impl Step for MergeReads {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        //todo:report more tahn one?
        if self.min_overlap < 5 {
            bail!("min_overlap must be >= 5. Set a valid value.");
        }
        if self.max_mismatch_rate < 0.0 || self.max_mismatch_rate >= 1.0 {
            bail!("max_mismatch_rate must be in [0.0..1.0). Set a valid value >= 0 and < 1.0.");
        }
        if let Some(space_quality_char) = self.spacer_quality_char
            && (!(33..=126).contains(&space_quality_char))
        {
            bail!("spacer_quality_char must be in [33..126]. Set a valid value.");
        }
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        self.out_label
            .as_ref()
            .map(|label| (label.clone(), crate::transformations::TagValueType::Bool))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let seg1_idx = self.segment1.0;
        let seg2_idx = self.segment2.0;
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
                    reads[seg1_idx].replace_seq(&merged_seq, &merged_qual);
                    // Clear segment2
                    reads[seg2_idx].clear();
                    true
                }
                MergeResult::NoOverlap => {
                    // Handle according to strategy
                    if no_overlap_strategy == NoOverlapStrategy::Concatenate {
                        let spacer = concatenate_spacer
                            .as_ref()
                            .expect("concatenate_spacer must be Some in concatenate mode");

                        // Concatenate read1 + spacer + read2_processed into segment1
                        let mut concatenated_seq = read1_seq.to_vec();
                        concatenated_seq.extend_from_slice(spacer.as_bytes());
                        concatenated_seq.extend_from_slice(&read2_seq_processed);

                        let mut concatenated_qual = read1_qual.to_vec();
                        concatenated_qual.extend(std::iter::repeat_n(spacer_qual, spacer.len()));
                        concatenated_qual.extend_from_slice(&read2_qual_processed);

                        // Update segment1 with concatenated sequence
                        reads[seg1_idx].replace_seq(&concatenated_seq, &concatenated_qual);
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
            block.tags.insert(
                self.out_label
                    .as_ref()
                    .expect("out_label must be set for merge type")
                    .clone(),
                tag_values,
            );
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
#[allow(clippy::too_many_arguments)]
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
    }
}

/// Find the best overlap using fastp algorithm (hamming distance)
/// I not fond of this. It's a faithful rewrite of the C(++) fastp code,
/// but it's missing a *large* set of test cases that verify that a)
/// it does what fastp actually does,
/// and b) that all of it's branches get exercised.
/// Mutation testing really is having a field day with this,
/// and devising test cases that cover all the branches & loop conditions
/// is somewhat tricky.
#[allow(clippy::cast_possible_truncation)] // u64 to usize is fine.
#[allow(clippy::cast_sign_loss)] // mas_mismatch_rate is 0..=1
#[allow(clippy::cast_precision_loss)] // mas_mismatch_rate is 0..=1
#[allow(clippy::nonminimal_bool)] 
fn find_best_overlap_fastp(
    seq1: &[u8],
    seq2: &[u8], //must already have been reverse complemented
    min_overlap: usize,
    max_mismatch_rate: f64,
    max_mismatch_count: usize,
) -> Option<(isize, usize)> {
    //offset, length
    //use bio::alignment::distance::hamming;
    let len1: isize = seq1.len().try_into().expect("seq1 len too large for isize");
    let len2: isize = seq2.len().try_into().expect("seq2 len too large for isize"); //already reverse complement

    let complete_compare_require = 50;

    let mut overlap_len;
    let mut offset: isize = 0;
    let mut diff;
    let overlap_require: isize = min_overlap.try_into().expect("min_overlap too large for isize");
    let diff_percent_limit = max_mismatch_rate;
    let diff_limit = max_mismatch_count;
    let str1 = seq1;
    let str2 = seq2;

    // forward
    // a match of less than overlapRequire is considered as unconfident
    while offset < len1.checked_sub(overlap_require).expect("Substraction below range") {
        // the overlap length of r1 & r2 when r2 is move right for offset
        overlap_len = (len1 - offset).min(len2);
        let overlap_diff_limit = diff_limit.min((overlap_len as f64 * diff_percent_limit) as usize);

        diff = 0;
        let mut last_i = 0;
        for i in 0..overlap_len {
            if str1[(offset + i) as usize] != str2[i as usize] {
                diff += 1;
                if diff > overlap_diff_limit && i < complete_compare_require {
                    break;
                }
            }
            last_i = i + 1;
        }
        if diff <= overlap_diff_limit
            || (diff > overlap_diff_limit && last_i > complete_compare_require)
        {
            return Some((offset, overlap_len as usize));
        }

        offset += 1;
    }

    // reverse
    // in this case, the adapter is sequenced since TEMPLATE_LEN < SEQ_LEN
    // check if distance can get smaller if offset goes negative
    // this only happens when insert DNA is shorter than sequencing read length, and some adapter/primer is sequenced but not trimmed cleanly
    // we go reversely
    offset = 0;
    while offset > -(len2 - overlap_require) {
        // the overlap length of r1 & r2 when r2 is move right for offset
        overlap_len = len1.min(len2 - (offset.abs()));
        let overlap_diff_limit = diff_limit.min((overlap_len as f64 * diff_percent_limit) as usize);

        diff = 0;
        let mut last_i = 0;
        for i in 0..overlap_len {
            if str1[i as usize] != str2[(-offset + i) as usize] {
                diff += 1;
                if diff > overlap_diff_limit && i < complete_compare_require {
                    break;
                }
            }
            last_i = i + 1;
        }

        if diff <= overlap_diff_limit
            || (diff > overlap_diff_limit && last_i > complete_compare_require)
        {
            return Some((offset, overlap_len as usize));
        }

        offset -= 1;
    }
    None
}

/// fastp is documented to prefer R1 bases, no matter what.
#[allow(clippy::cast_sign_loss)]
fn merge_at_offset_fastp(
    seq1: &[u8],
    qual1: &[u8],
    seq2: &[u8],
    qual2: &[u8],
    offset: isize,
    overlap_len: usize,
) -> Result<(Vec<u8>, Vec<u8>)> {
    #[allow(clippy::too_many_arguments)]
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
                if q2 >= GOOD_QUAL && q1 <= BAD_QUAL {
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

        // Non-overlapping part of seq2 - ONLY if offset > 0! to match fastp
        if offset > 0 && overlap_len < seq2.len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_best_overlap_fastp() {
        let seq1 = b"ACGTACGTACGT";
        let seq2 = b"GTACGTACGTAA";

        let result = find_best_overlap_fastp(seq1, seq2, 4, 0.2, 2);
        assert_eq!(result, Some((2, 10))); // seq2 starts at position 4 in seq1 with overlap length 8

        let result = find_best_overlap_fastp(b"AGTCAA", b"CTCCA", 4, 0.2, 2);
        assert_eq!(result, None); // No sufficient overlap

        let result = find_best_overlap_fastp(b"AGTCAA", b"AGTCAA", 4, 0.2, 2);
        assert_eq!(result, Some((0, 6))); // Perfect overlap
        //
        let result = find_best_overlap_fastp(b"AGTCAA", b"ACAGTCAA", 4, 0.2, 2);
        assert_eq!(result, Some((-2, 6)));

        //good threshold is ?
        //bad threshold is /

        let r = merge_at_offset_fastp(
            b"AAAAAAAAAAAAAAAAAA",
            b"???@@@>>>./0./0./0",
            b"TTTTTTTTTTTTTTTTTT",
            b"./0./0./0???@@@>>>",
            0,
            18,
        )
        .expect("merge failed");
        assert_eq!(&r.0, b"AAAAAAAAATTATTAAAA");
        //assert_eq!(&r.1, b"cccc");
    }
}
