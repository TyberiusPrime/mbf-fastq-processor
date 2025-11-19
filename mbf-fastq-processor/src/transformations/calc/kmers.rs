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
    /// Tag name to store the result
    pub out_label: String,
    /// Any of your input segments, or 'All'
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,

    /// Sequence files to build kmer database from
    #[serde(deserialize_with = "deser::string_or_seq")]
    #[serde(alias = "filename")]
    pub files: Vec<String>,
    /// Kmer length
    pub k: usize,
    /// Whether to also include each revcomp of a kmer in the database ('canonical kmers')
    #[serde(alias = "canonical")]
    pub count_reverse_complement: bool,
    /// Minimum occurrences (forward+reverse if count_reverse_complement is set) in reference to include kmer
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

        super::extract_numeric_tags_plus_all(
            self.segment_index.unwrap(),
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
