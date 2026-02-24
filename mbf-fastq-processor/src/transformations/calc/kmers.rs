#![allow(clippy::unnecessary_wraps)]

use crate::io;
use crate::transformations::prelude::*;
use std::collections::HashMap;

fn default_min_count() -> usize {
    1
}

/// Quantify Kmer occurance vs database
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Kmers {
    pub out_label: String,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,

    // Kmer database configuration
    #[tpd(alias = "files")]
    #[tpd(alias = "filenames")]
    pub filename: Vec<String>,

    pub k: usize,

    #[tpd(alias = "canonical")]
    pub count_reverse_complement: bool,

    pub min_count: usize,

    #[schemars(skip)]
    #[tpd(skip, default)]
    pub resolved_kmer_db: Option<HashMap<Vec<u8>, usize>>,
}

impl VerifyIn<PartialConfig> for PartialKmers {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.segment.validate_segment(parent);
        self.min_count.or_with(default_min_count);

        self.filename.verify(|filenames| {
            if filenames.is_empty() {
                return Err(ValidationFailure::new(
                    "At least one filename must be provided for the k-mer database.",
                    Some("Please specify the path to your k-mer database file."),
                ));
            }
            if filenames.iter().any(|filepath| filepath.as_ref().expect("Should not be reached on wrong type for filename") == crate::config::STDIN_MAGIC_PATH) {
                return Err(ValidationFailure::new(
                    "QuantifyKmers: K-mer database cannot be read from stdin",
                    Some("Please specify a file path for the k-mer database instead of using '-' or 'stdin'.")
                ));
            }
            Ok(())
        });

        self.k.verify(|k| {
            if *k == 0 {
                return Err(ValidationFailure::new(
                    "'k' must be greater than 0.",
                    Some("Please specify a positive integer value for k (e.g., k = 5 for 5-mers)."),
                ));
            }
            Ok(())
        });

        Ok(())
    }
}

impl Step for Kmers {
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
            &self.filename,
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
        &self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let kmer_db = self
            .resolved_kmer_db
            .as_ref()
            .expect("resolved_kmer_db must be set during initialization");
        let k = self.k;

        super::extract_numeric_tags_plus_all(
            self.segment,
            &self.out_label,
            #[allow(clippy::cast_precision_loss)]
            |read| {
                let count = count_kmers_in_database(read.seq(), k, kmer_db);
                count as f64
            },
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let total_count: usize = reads
                    .iter()
                    .map(|read| count_kmers_in_database(read.seq(), k, kmer_db))
                    .sum();
                total_count as f64
            },
            &mut block,
        );

        Ok((block, true))
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
                            if canonical {
                                let revcomp = crate::dna::reverse_complement(&kmer);
                                *kmer_counts.entry(revcomp).or_insert(0) += 1;
                            }
                            *kmer_counts.entry(kmer).or_insert(0) += 1;
                        }
                    }
                }
            },
            true,
            true, //all reads in BAM.
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
