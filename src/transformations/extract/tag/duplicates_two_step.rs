#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use std::{collections::HashMap, path::Path};

use crate::config::SegmentIndexOrAll;
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use crate::transformations::{
    FragmentEntry, InputInfo, RegionDefinition, Step, extract_regions, reproducible_cuckoofilter,
};
use serde_valid::Validate;

use super::duplicates::ApproxOrExactFilter;

/// Two-step duplicate detection:
/// - First step: exact matching on first_region using HashMap
/// - Second step: probabilistic matching on second_regions using CuckooFilter (per first_region value)
#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct DuplicatesTwoStep {
    pub label: String,

    pub first_region: RegionDefinition,

    #[validate(min_items = 1)]
    pub second_regions: Vec<RegionDefinition>,

    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,

    pub seed: Option<u64>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filters: Option<HashMap<Vec<u8>, ApproxOrExactFilter>>,
}

impl DuplicatesTwoStep {
    fn estimate_capacity(&self) -> usize {
        // Estimate max instances as 4^n where n is total length of second_regions
        let total_length: usize = self.second_regions.iter().map(|r| r.length).sum();
        // Cap at a reasonable maximum to avoid overflow
        let capped_length = total_length.min(15); // 4^15 = ~1 billion
        4_usize.pow(capped_length as u32)
    }
}

impl Step for DuplicatesTwoStep {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate seed requirement based on false_positive_rate
        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        // Validate first_region
        crate::transformations::validate_regions(
            std::slice::from_mut(&mut self.first_region),
            input_def,
        )?;

        // Validate second_regions
        crate::transformations::validate_regions(&mut self.second_regions, input_def)?;

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        self.filters = Some(HashMap::new());
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        // Extract parameters before borrowing filters
        let false_positive_rate = self.false_positive_rate;
        let seed = self.seed;
        let estimated_capacity = self.estimate_capacity();

        // Now we can safely borrow filters mutably
        let filters = std::sync::Arc::new(std::sync::Mutex::new(self.filters.as_mut().unwrap()));

        // Create tags if not present
        if block.tags.is_none() {
            block.tags = Some(HashMap::new());
        }

        let mut tag_values = Vec::new();

        for read_idx in 0..block.len() {
            // Extract first region
            let first_seq_parts =
                extract_regions(read_idx, &block, std::slice::from_ref(&self.first_region));
            let first_seq: Vec<u8> = first_seq_parts
                .into_iter()
                .flat_map(|s| s.to_vec())
                .collect();

            // Extract second regions
            let second_seq_parts = extract_regions(read_idx, &block, &self.second_regions);
            let second_seq_refs: Vec<&[u8]> = second_seq_parts.iter().map(|s| s.as_ref()).collect();
            let second_entry = FragmentEntry(&second_seq_refs);

            // Get or create filter for this first_region value
            let mut filters_guard = filters.lock().unwrap();
            let filter = filters_guard.entry(first_seq).or_insert_with(|| {
                if false_positive_rate == 0.0 {
                    ApproxOrExactFilter::Exact(std::collections::HashSet::new())
                } else {
                    let seed_val = seed
                        .expect("seed should be validated to exist when false_positive_rate > 0.0");
                    ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                        seed_val,
                        estimated_capacity,
                        false_positive_rate,
                    )))
                }
            });

            // Check if this second region sequence was seen before for this first region
            let is_duplicate = filter.containsert(&second_entry);
            drop(filters_guard);

            tag_values.push(crate::dna::TagValue::Bool(is_duplicate));
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label.clone(), tag_values);

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        true // We need serial processing because we're maintaining state
    }
}
