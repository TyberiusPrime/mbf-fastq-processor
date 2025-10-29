#![allow(clippy::unnecessary_wraps)]
use crate::config::{KmerDb, SegmentIndexOrAll, SegmentOrAll};
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::path::Path;

use super::super::Step;
use crate::{Demultiplexed, kmer};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QuantifyKmers {
    pub label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
    // Reference to kmer_db section
    pub kmer_db: String,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub resolved_kmer_db: Option<HashMap<Vec<u8>, usize>>,
    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub k: usize,
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
        _barcodes: &std::collections::HashMap<String, crate::config::Barcodes>,
        kmer_dbs: &std::collections::HashMap<String, KmerDb>,
    ) -> Result<()> {
        // Resolve the kmer_db reference
        match kmer_dbs.get(&self.kmer_db) {
            Some(kmer_db_config) => {
                // Build the kmer database from files
                let db = kmer::build_kmer_database(
                    &kmer_db_config.files,
                    kmer_db_config.k,
                    kmer_db_config.min_count,
                )?;
                self.resolved_kmer_db = Some(db);
                self.k = kmer_db_config.k;
            }
            None => {
                bail!(
                    "Kmer database section '{}' not found. Available sections: {:?}",
                    self.kmer_db,
                    kmer_dbs.keys().collect::<Vec<_>>()
                );
            }
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
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
