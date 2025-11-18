#![allow(clippy::unnecessary_wraps)]
use std::collections::HashMap;
use std::path::Path;

use crate::transformations::prelude::*;
use crate::{config::deser, io};

fn default_min_count() -> usize {
    1
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Kmers {
    pub out_label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,

    // Kmer database configuration
    #[serde(deserialize_with = "deser::string_or_seq")]
    #[serde(alias = "filename")]
    pub files: Vec<String>,
    pub k: usize,
    #[serde(alias = "canonical")]
    pub count_reverse_complement: bool,
    #[serde(default = "default_min_count")]
    pub min_count: usize,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub resolved_kmer_db: Option<HashMap<Vec<u8>, usize>>,
}

impl Step for Kmers {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.files.is_empty() {
            bail!("QuantifyKmers: 'files' must contain at least one file");
        }
        if self.k == 0 {
            bail!("QuantifyKmers: 'k' must be greater than 0");
        }
        // Check that files exist (will be checked again at runtime, but helpful to fail early)
        if self
            .files
            .iter()
            .any(|filepath| filepath == crate::config::STDIN_MAGIC_PATH)
        {
            bail!("KMer database can't be read from stdin. Sorry");
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let db = build_kmer_database(
            &self.files,
            self.k,
            self.min_count,
            self.count_reverse_complement,
        )?;
        self.resolved_kmer_db = Some(db);

        Ok(None)
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let kmer_db = self.resolved_kmer_db.as_ref().unwrap();
        let k = self.k;
        let canonical = self.count_reverse_complement;

        super::extract_numeric_tags_plus_all(
            self.segment_index.unwrap(),
            &self.out_label,
            #[allow(clippy::cast_precision_loss)]
            |read| {
                let count = count_kmers_in_database(read.seq(), k, kmer_db, canonical);
                count as f64
            },
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let total_count: usize = reads
                    .iter()
                    .map(|read| count_kmers_in_database(read.seq(), k, kmer_db, canonical))
                    .sum();
                total_count as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}

/// Get the canonical form of a kmer (lexicographically smaller of forward and reverse complement)
fn canonical_kmer(kmer: &[u8]) -> Vec<u8> {
    let revcomp = crate::dna::reverse_complement(kmer);
    if kmer <= revcomp.as_slice() {
        kmer.to_vec()
    } else {
        revcomp
    }
}

pub fn build_kmer_database(
    files: &[String],
    k: usize,
    min_count: usize,
    canonical: bool,
) -> Result<HashMap<Vec<u8>, usize>> {
    let mut kmer_counts: HashMap<Vec<u8>, usize> = HashMap::new();

    for file_path in files {
        io::apply_to_read_sequences(
            file_path,
            &mut |seq: &[u8]| {
                // Extract all kmers from this sequence
                if seq.len() >= k {
                    for i in 0..=(seq.len() - k) {
                        let kmer: Vec<u8> = seq[i..i + k]
                            .iter()
                            .map(|&b| b.to_ascii_uppercase())
                            .collect();

                        // Only count valid DNA sequences (A, C, G, T)
                        if kmer.iter().all(|&b| matches!(b, b'A' | b'C' | b'G' | b'T')) {
                            let kmer_to_store = if canonical {
                                canonical_kmer(&kmer)
                            } else {
                                kmer
                            };
                            *kmer_counts.entry(kmer_to_store).or_insert(0) += 1;
                        }
                    }
                }
            },
            None, // Don't ignore unmapped reads
        )
        .with_context(|| format!("Failed to parse kmer database file: {file_path}"))?;
    }

    // Filter by minimum count
    kmer_counts.retain(|_, &mut count| count >= min_count);

    Ok(kmer_counts)
}

/// Count how many kmers from a sequence are in the database
pub fn count_kmers_in_database(
    sequence: &[u8],
    k: usize,
    kmer_db: &HashMap<Vec<u8>, usize>,
    canonical: bool,
) -> usize {
    if sequence.len() < k {
        return 0;
    }

    let mut count = 0;
    for i in 0..=(sequence.len() - k) {
        let kmer: Vec<u8> = sequence[i..i + k]
            .iter()
            .map(|&b| b.to_ascii_uppercase())
            .collect();

        let lookup_kmer = if canonical {
            canonical_kmer(&kmer)
        } else {
            kmer
        };

        if kmer_db.contains_key(&lookup_kmer) {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_kmer_basic() {
        // AAAC < GTTT (forward is lexicographically smaller)
        assert_eq!(canonical_kmer(b"AAAC"), b"AAAC");

        // CGTT > AACG (reverse complement is lexicographically smaller)
        assert_eq!(canonical_kmer(b"CGTT"), b"AACG");

        // ACGT is a palindrome
        assert_eq!(canonical_kmer(b"ACGT"), b"ACGT");
    }

    #[test]
    fn test_canonical_kmer_orientation() {
        // Test that forward and reverse complement give same canonical form
        let forward = b"AAACGTTT";
        let revcomp = crate::dna::reverse_complement(forward);

        let canonical_fwd = canonical_kmer(forward);
        let canonical_rev = canonical_kmer(&revcomp);

        assert_eq!(
            canonical_fwd, canonical_rev,
            "Forward and reverse complement should have same canonical form"
        );
    }

    #[test]
    fn test_build_kmer_database_canonical() {
        // Create a temporary file with a simple sequence
        let temp_file = "/tmp/test_kmer_canonical.fa";
        // Sequence contains AAAA (canonical) and its reverse TTTT
        std::fs::write(temp_file, ">test\nAAAACGTT\n").unwrap();

        let db = build_kmer_database(&[temp_file.to_string()], 4, 1, true).unwrap();

        // With canonical=true, we should have fewer entries than with canonical=false
        // AAAA and TTTT should map to the same canonical kmer
        println!("Database entries:");
        for (kmer, count) in &db {
            println!("  {}: {}", String::from_utf8_lossy(kmer), count);
        }

        // AAAA is canonical (AAAA < TTTT)
        assert!(
            db.contains_key(b"AAAA"),
            "Database should contain AAAA (canonical form)"
        );

        // TTTT should NOT be in the database (it's the non-canonical form of AAAA)
        assert!(
            !db.contains_key(b"TTTT"),
            "Database should NOT contain TTTT (non-canonical form)"
        );

        // The canonical kmer AAAA should have count=1 (appeared once in forward direction)
        assert_eq!(db.get(b"AAAA"), Some(&1));

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_build_kmer_database_non_canonical() {
        let temp_file = "/tmp/test_kmer_non_canonical.fa";
        std::fs::write(temp_file, ">test\nAAAACGTT\n").unwrap();

        let db = build_kmer_database(&[temp_file.to_string()], 4, 1, false).unwrap();

        // With canonical=false, we should have the exact kmers as they appear
        assert!(
            db.contains_key(b"AAAA"),
            "Database should contain AAAA (as it appears)"
        );
        assert!(
            !db.contains_key(b"TTTT"),
            "Database should NOT contain TTTT (doesn't appear in sequence)"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_canonical_counts_both_orientations() {
        let temp_file = "/tmp/test_kmer_both_orientations.fa";
        // This sequence contains AAAC (forward) and GTTT (reverse complement) at different positions
        std::fs::write(temp_file, ">test\nAAACNNNNGTTT\n").unwrap();

        let db = build_kmer_database(&[temp_file.to_string()], 4, 1, true).unwrap();

        // Both AAAC and GTTT should map to the canonical form AAAC
        // So the count should be 2
        let canonical_form = canonical_kmer(b"AAAC");
        assert_eq!(
            db.get(&canonical_form),
            Some(&2),
            "Canonical kmer should count both orientations"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_count_kmers_in_database_canonical() {
        let temp_file = "/tmp/test_kmer_query_canonical.fa";
        std::fs::write(temp_file, ">test\nAAAA\n").unwrap();

        let db = build_kmer_database(&[temp_file.to_string()], 4, 1, true).unwrap();

        // Query with forward orientation
        let count_forward = count_kmers_in_database(b"AAAA", 4, &db, true);
        assert_eq!(count_forward, 1, "Forward kmer should match");

        // Query with reverse complement orientation (TTTT is revcomp of AAAA)
        let count_reverse = count_kmers_in_database(b"TTTT", 4, &db, true);
        assert_eq!(
            count_reverse, 1,
            "Reverse complement should match when using canonical"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_count_kmers_in_database_non_canonical() {
        let temp_file = "/tmp/test_kmer_query_non_canonical.fa";
        std::fs::write(temp_file, ">test\nAAAA\n").unwrap();

        let db = build_kmer_database(&[temp_file.to_string()], 4, 1, false).unwrap();

        // Query with forward orientation
        let count_forward = count_kmers_in_database(b"AAAA", 4, &db, false);
        assert_eq!(count_forward, 1, "Forward kmer should match");

        // Query with reverse complement orientation should NOT match
        let count_reverse = count_kmers_in_database(b"TTTT", 4, &db, false);
        assert_eq!(
            count_reverse, 0,
            "Reverse complement should NOT match when not using canonical"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_min_count_with_canonical() {
        let temp_file = "/tmp/test_kmer_min_count_canonical.fa";
        // Sequence with AAAC appearing twice in forward, and GTTT (its revcomp) appearing twice
        // Total canonical count should be 4
        std::fs::write(temp_file, ">test\nAAACAAACGTTTGTTT\n").unwrap();

        // With min_count=3, the canonical kmer (count=4) should be kept
        let db = build_kmer_database(&[temp_file.to_string()], 4, 3, true).unwrap();

        let canonical_form = canonical_kmer(b"AAAC");
        assert!(
            db.contains_key(&canonical_form),
            "Canonical kmer with count=4 should pass min_count=3"
        );
        assert_eq!(db.get(&canonical_form), Some(&4));

        // With min_count=5, it should be filtered out
        let db_filtered = build_kmer_database(&[temp_file.to_string()], 4, 5, true).unwrap();
        assert!(
            !db_filtered.contains_key(&canonical_form),
            "Canonical kmer with count=4 should not pass min_count=5"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_database_size_reduction() {
        let temp_file = "/tmp/test_kmer_size_reduction.fa";
        // PhiX-like sequence (a portion of it)
        std::fs::write(
            temp_file,
            ">test\nGAGTTTTATCGCTTCCATGACGCAGAAGTTAACACTTTCGGATATTTCTGATGAGTCGAAAAATTATCTT\n",
        )
        .unwrap();

        let db_canonical = build_kmer_database(&[temp_file.to_string()], 21, 1, true).unwrap();
        let db_non_canonical =
            build_kmer_database(&[temp_file.to_string()], 21, 1, false).unwrap();

        println!("Canonical database size: {}", db_canonical.len());
        println!("Non-canonical database size: {}", db_non_canonical.len());

        // Canonical database should be approximately 50% the size (or less due to palindromes)
        assert!(
            db_canonical.len() <= db_non_canonical.len(),
            "Canonical database should not be larger than non-canonical"
        );

        std::fs::remove_file(temp_file).ok();
    }
}
