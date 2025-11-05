#![allow(clippy::unnecessary_wraps)]
use std::collections::HashMap;

use crate::transformations::prelude::*;

use crate::kmer;

fn default_min_count() -> usize {
    1
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QuantifyKmers {
    pub label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,

    // Kmer database configuration
    pub files: Vec<String>,
    pub k: usize,
    #[serde(default = "default_min_count")]
    pub min_count: usize,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub resolved_kmer_db: Option<HashMap<Vec<u8>, usize>>,
}

impl Step for QuantifyKmers {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn resolve_config_references(
        &mut self,
        _barcodes: &std::collections::BTreeMap<String, crate::config::Barcodes>,
    ) -> Result<()> {
        // Build the kmer database from files
        let db = kmer::build_kmer_database(&self.files, self.k, self.min_count)?;
        self.resolved_kmer_db = Some(db);
        Ok(())
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
            &self.label,
            #[allow(clippy::cast_precision_loss)]
            |read| {
                let count = kmer::count_kmers_in_database(read.seq(), k, kmer_db);
                count as f64
            },
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let total_count: usize = reads
                    .iter()
                    .map(|read| kmer::count_kmers_in_database(read.seq(), k, kmer_db))
                    .sum();
                total_count as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}
