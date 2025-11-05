use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::io;

/// Build a kmer database from sequence files
///
/// TODO: Add support for canonical kmers (treating a kmer and its reverse complement
/// as equivalent). This is the bioinformatics best practice since DNA is double-stranded.
/// Should add a `canonical: bool` parameter (defaulting to true) that, when enabled,
/// stores only the lexicographically smaller of each kmer/revcomp pair.
pub fn build_kmer_database(
    files: &[String],
    k: usize,
    min_count: usize,
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
                            *kmer_counts.entry(kmer).or_insert(0) += 1;
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

        if kmer_db.contains_key(&kmer) {
            count += 1;
        }
    }

    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_kmers() {
        let mut db = HashMap::new();
        db.insert(b"ATG".to_vec(), 1);
        db.insert(b"TGC".to_vec(), 1);

        let seq = b"ATGC";
        let count = count_kmers_in_database(seq, 3, &db);
        assert_eq!(count, 2); // ATG and TGC are both in the database
    }

    #[test]
    fn test_count_kmers_case_insensitive() {
        let mut db = HashMap::new();
        db.insert(b"ATG".to_vec(), 1);

        let seq = b"atg";
        let count = count_kmers_in_database(seq, 3, &db);
        assert_eq!(count, 1); // Should match case-insensitively
    }
}
