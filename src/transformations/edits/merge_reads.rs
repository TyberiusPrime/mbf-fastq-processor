#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use crate::config::SegmentIndex;

/// Strategy when reads cannot be merged due to insufficient overlap
#[derive(eserde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoOverlapStrategy {
    /// Keep reads as they are (no merging)
    Keep,
    /// Concatenate reads with optional spacer into first segment
    Concatenate,
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct MergeReads {
    /// Minimum overlap length required for merging (suggested: 30)
    pub min_overlap: usize,

    /// Maximum allowed mismatch rate (0.0 to 1.0, suggested: 0.2)
    pub max_mismatch_rate: f64,

    /// Allow single gap (insertion/deletion) during alignment (suggested: false)
    #[serde(default)]
    pub allow_gap: bool,

    /// Strategy when no overlap is found (suggested: keep)
    #[serde(default)]
    pub no_overlap_strategy: Option<NoOverlapStrategy>,

    /// Spacer sequence to use when concatenating (required if no_overlap_strategy = 'concatenate')
    #[serde(default)]
    pub concatenate_spacer: Option<String>,

    /// Quality score to use for spacer bases (suggested: 33, which is Phred quality 0)
    #[serde(default)]
    pub spacer_quality_char: Option<u8>,

    /// First segment (typically read1, suggested: "read1")
    #[serde(default)]
    pub segment1: Option<String>,
    #[serde(default)]
    #[serde(skip)]
    pub segment1_index: Option<SegmentIndex>,

    /// Second segment (typically read2, suggested: "read2")
    #[serde(default)]
    pub segment2: Option<String>,
    #[serde(default)]
    #[serde(skip)]
    pub segment2_index: Option<SegmentIndex>,
}

impl Step for MergeReads {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        // Provide defaults if not specified
        let segment1_name = self.segment1.clone().unwrap_or_else(|| "read1".to_string());
        let segment2_name = self.segment2.clone().unwrap_or_else(|| "read2".to_string());

        // Validate segment1
        let mut seg1 = crate::config::Segment(segment1_name);
        self.segment1_index = Some(seg1.validate(input_def)?);

        // Validate segment2
        let mut seg2 = crate::config::Segment(segment2_name);
        self.segment2_index = Some(seg2.validate(input_def)?);

        // Ensure they're different segments
        if self.segment1_index == self.segment2_index {
            bail!("segment1 and segment2 must be different segments");
        }

        // Validate parameters
        if self.min_overlap == 0 {
            bail!("min_overlap must be > 0");
        }

        if !(0.0..=1.0).contains(&self.max_mismatch_rate) {
            bail!("max_mismatch_rate must be between 0.0 and 1.0");
        }

        // Validate concatenate_spacer requirement
        let strategy = self.no_overlap_strategy.clone().unwrap_or(NoOverlapStrategy::Keep);
        if strategy == NoOverlapStrategy::Concatenate && self.concatenate_spacer.is_none() {
            bail!("concatenate_spacer is required when no_overlap_strategy = 'concatenate'");
        }

        // Validate spacer_quality_char
        if let Some(qual_char) = self.spacer_quality_char {
            if qual_char < 33 || qual_char > 126 {
                bail!("spacer_quality_char must be a valid ASCII character in range 33-126");
            }
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

        let num_reads = block.segments[seg1_idx].entries.len();

        // Process each read pair
        for i in 0..num_reads {
            // Get references to both segments
            let seg1_block = &mut block.segments[seg1_idx];
            let read1_seq = seg1_block.entries[i].seq.get(&seg1_block.block).to_vec();
            let read1_qual = seg1_block.entries[i].qual.get(&seg1_block.block).to_vec();

            let seg2_block = &mut block.segments[seg2_idx];
            let read2_seq = seg2_block.entries[i].seq.get(&seg2_block.block).to_vec();
            let read2_qual = seg2_block.entries[i].qual.get(&seg2_block.block).to_vec();

            // Reverse complement read2 for proper merging
            let read2_seq_rc = reverse_complement(&read2_seq);
            let read2_qual_rc: Vec<u8> = read2_qual.iter().rev().copied().collect();

            // Try to find overlap and merge
            let merge_result =
                self.try_merge_reads(&read1_seq, &read1_qual, &read2_seq_rc, &read2_qual_rc)?;

            // Apply the merge result
            match merge_result {
                MergeResult::Merged {
                    merged_seq,
                    merged_qual,
                } => {
                    // Update segment1 with merged sequence
                    let seg1_block = &mut block.segments[seg1_idx];
                    seg1_block.entries[i]
                        .seq
                        .replace(merged_seq, &mut seg1_block.block);
                    seg1_block.entries[i]
                        .qual
                        .replace(merged_qual, &mut seg1_block.block);

                    // Clear segment2
                    let seg2_block = &mut block.segments[seg2_idx];
                    seg2_block.entries[i]
                        .seq
                        .replace(Vec::new(), &mut seg2_block.block);
                    seg2_block.entries[i]
                        .qual
                        .replace(Vec::new(), &mut seg2_block.block);
                }
                MergeResult::NoOverlap => {
                    // Handle according to strategy
                    let strategy = self.no_overlap_strategy.clone().unwrap_or(NoOverlapStrategy::Keep);
                    if strategy == NoOverlapStrategy::Concatenate {
                        let spacer = self.concatenate_spacer.as_ref().unwrap();
                        let spacer_qual = self.spacer_quality_char.unwrap_or(33);

                        // Concatenate read1 + spacer + read2_rc into segment1
                        let mut concatenated_seq = read1_seq.clone();
                        concatenated_seq.extend_from_slice(spacer.as_bytes());
                        concatenated_seq.extend_from_slice(&read2_seq_rc);

                        let mut concatenated_qual = read1_qual.clone();
                        concatenated_qual.extend(
                            std::iter::repeat(spacer_qual)
                                .take(spacer.len()),
                        );
                        concatenated_qual.extend_from_slice(&read2_qual_rc);

                        // Update segment1 with concatenated sequence
                        let seg1_block = &mut block.segments[seg1_idx];
                        seg1_block.entries[i]
                            .seq
                            .replace(concatenated_seq, &mut seg1_block.block);
                        seg1_block.entries[i]
                            .qual
                            .replace(concatenated_qual, &mut seg1_block.block);

                        // Clear segment2
                        let seg2_block = &mut block.segments[seg2_idx];
                        seg2_block.entries[i]
                            .seq
                            .replace(Vec::new(), &mut seg2_block.block);
                        seg2_block.entries[i]
                            .qual
                            .replace(Vec::new(), &mut seg2_block.block);
                    }
                    // Otherwise keep reads as they are (NoOverlapStrategy::Keep)
                }
            }
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

impl MergeReads {
    /// Try to merge two reads by finding their overlap
    /// seq2 and qual2 should already be reverse complemented
    /// Returns the merged sequence and quality if successful
    fn try_merge_reads(
        &self,
        seq1: &[u8],
        qual1: &[u8],
        seq2_rc: &[u8],
        qual2_rc: &[u8],
    ) -> Result<MergeResult> {
        // Try to find the best overlap
        let overlap = self.find_best_overlap(seq1, qual1, seq2_rc, qual2_rc);

        if let Some((offset, overlap_len)) = overlap {
            // Merge the reads
            let (merged_seq, merged_qual) =
                self.merge_at_offset(seq1, qual1, seq2_rc, qual2_rc, offset, overlap_len)?;
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
    fn find_best_overlap(
        &self,
        seq1: &[u8],
        _qual1: &[u8],
        seq2_rc: &[u8],
        _qual2_rc: &[u8],
    ) -> Option<(isize, usize)> {
        let len1 = seq1.len();
        let len2 = seq2_rc.len();

        let mut best_match: Option<(isize, usize, usize)> = None; // (offset, overlap_len, mismatches)

        // Phase 1: Forward alignment (seq2_rc starts inside seq1)
        // offset is the position in seq1 where seq2_rc starts
        let max_offset = len1.saturating_sub(self.min_overlap);
        for offset in 0..=max_offset {
            let overlap_len = (len1 - offset).min(len2);
            if overlap_len < self.min_overlap {
                break;
            }

            let mismatches =
                count_mismatches(&seq1[offset..offset + overlap_len], &seq2_rc[..overlap_len]);
            let max_allowed_mismatches =
                (overlap_len as f64 * self.max_mismatch_rate).floor() as usize;

            if mismatches <= max_allowed_mismatches {
                if best_match.is_none() || mismatches < best_match.unwrap().2 {
                    best_match = Some((offset as isize, overlap_len, mismatches));
                }
            }
        }

        // Phase 2: Reverse alignment (seq1 starts inside seq2_rc)
        // negative offset means seq2_rc starts before seq1
        let max_offset = len2.saturating_sub(self.min_overlap);
        for offset in 1..=max_offset {
            let overlap_len = (len2 - offset).min(len1);
            if overlap_len < self.min_overlap {
                break;
            }

            let mismatches =
                count_mismatches(&seq2_rc[offset..offset + overlap_len], &seq1[..overlap_len]);
            let max_allowed_mismatches =
                (overlap_len as f64 * self.max_mismatch_rate).floor() as usize;

            if mismatches <= max_allowed_mismatches {
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
        &self,
        seq1: &[u8],
        qual1: &[u8],
        seq2_rc: &[u8],
        qual2_rc: &[u8],
        offset: isize,
        overlap_len: usize,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut merged_seq = Vec::new();
        let mut merged_qual = Vec::new();

        if offset >= 0 {
            let offset = offset as usize;
            // seq2_rc starts at position 'offset' in seq1
            // merged = seq1[0..offset] + merge(overlap) + seq2_rc[overlap_len..]

            // Non-overlapping part of seq1
            merged_seq.extend_from_slice(&seq1[..offset]);
            merged_qual.extend_from_slice(&qual1[..offset]);

            // Overlapping part - resolve mismatches using quality scores
            for i in 0..overlap_len {
                let pos1 = offset + i;
                let pos2 = i;

                let base1 = seq1[pos1];
                let base2 = seq2_rc[pos2];
                let q1 = qual1[pos1];
                let q2 = qual2_rc[pos2];

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

            // Non-overlapping part of seq2_rc
            if overlap_len < seq2_rc.len() {
                merged_seq.extend_from_slice(&seq2_rc[overlap_len..]);
                merged_qual.extend_from_slice(&qual2_rc[overlap_len..]);
            }
        } else {
            let offset = (-offset) as usize;
            // seq1 starts at position 'offset' in seq2_rc
            // merged = seq2_rc[0..offset] + merge(overlap) + seq1[overlap_len..]

            // Non-overlapping part of seq2_rc
            merged_seq.extend_from_slice(&seq2_rc[..offset]);
            merged_qual.extend_from_slice(&qual2_rc[..offset]);

            // Overlapping part - resolve mismatches using quality scores
            for i in 0..overlap_len {
                let pos1 = i;
                let pos2 = offset + i;

                let base1 = seq1[pos1];
                let base2 = seq2_rc[pos2];
                let q1 = qual1[pos1];
                let q2 = qual2_rc[pos2];

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
}

/// Reverse complement a DNA sequence
fn reverse_complement(seq: &[u8]) -> Vec<u8> {
    seq.iter()
        .rev()
        .map(|&base| match base {
            b'A' => b'T',
            b'T' => b'A',
            b'C' => b'G',
            b'G' => b'C',
            b'N' => b'N',
            // Handle lowercase as well
            b'a' => b't',
            b't' => b'a',
            b'c' => b'g',
            b'g' => b'c',
            b'n' => b'n',
            _ => base, // Pass through other characters
        })
        .collect()
}

/// Count mismatches between two sequences
fn count_mismatches(seq1: &[u8], seq2: &[u8]) -> usize {
    seq1.iter().zip(seq2.iter()).filter(|(a, b)| a != b).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_complement() {
        assert_eq!(reverse_complement(b"ATCG"), b"CGAT");
        assert_eq!(reverse_complement(b"AAAA"), b"TTTT");
        assert_eq!(reverse_complement(b"GCGC"), b"GCGC");
        assert_eq!(reverse_complement(b"ATCGN"), b"NCGAT");
    }

    #[test]
    fn test_count_mismatches() {
        assert_eq!(count_mismatches(b"ATCG", b"ATCG"), 0);
        assert_eq!(count_mismatches(b"ATCG", b"ATGG"), 1);
        assert_eq!(count_mismatches(b"ATCG", b"GGGG"), 3);
        assert_eq!(count_mismatches(b"AAAA", b"TTTT"), 4);
    }

    #[test]
    fn test_find_overlap_perfect_match() {
        let merger = MergeReads {
            min_overlap: 5,
            max_mismatch_rate: 0.0,
            allow_gap: false,
            no_overlap_strategy: Some(NoOverlapStrategy::Keep),
            concatenate_spacer: None,
            spacer_quality_char: Some(33),
            segment1: Some("read1".to_string()),
            segment1_index: None,
            segment2: Some("read2".to_string()),
            segment2_index: None,
        };

        // seq1:     ATCGATCGATCG
        // seq2_rc:       ATCGATCGNNNN (rc of NNNNGATCGAT)
        // overlap at offset 4, length 8
        let seq1 = b"ATCGATCGATCG";
        let seq2_rc = b"ATCGATCGNNNN";
        let qual1 = vec![b'I'; seq1.len()];
        let qual2_rc = vec![b'I'; seq2_rc.len()];

        let result = merger.find_best_overlap(seq1, &qual1, seq2_rc, &qual2_rc);
        assert!(result.is_some());
        let (offset, overlap_len) = result.unwrap();
        assert_eq!(offset, 4);
        assert_eq!(overlap_len, 8);
    }
}
