use crate::transformations::prelude::*;

use super::super::validate_dna;
use super::common::default_true;
use crate::config::{SegmentIndexOrAll, SegmentOrAll};
use std::collections::HashSet;
use std::path::Path;

use super::super::tag::default_segment_all;

#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CountingStrategy {
    /// Original implementation - separate passes for base and quality counting
    Original,
    /// Optimized quality counting using lookup tables and accumulators
    Optimized,
    /// Merged single-pass counting for both bases and qualities
    Merged,
}

impl Default for CountingStrategy {
    fn default() -> Self {
        CountingStrategy::Original
    }
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Report {
    pub name: String,
    #[serde(default = "default_true")]
    pub count: bool,
    #[serde(default)]
    pub base_statistics: bool,
    #[serde(default)]
    pub length_distribution: bool,
    #[serde(default)]
    pub duplicate_count_per_read: bool,
    #[serde(default)]
    pub duplicate_count_per_fragment: bool,

    #[serde(default)]
    #[schemars(skip)]
    pub debug_reproducibility: bool,

    #[serde(default)]
    pub count_oligos: Option<Vec<String>>,
    #[serde(default = "default_segment_all")]
    pub count_oligos_segment: SegmentOrAll,

    #[serde(default)]
    #[serde(skip)]
    #[schemars(skip)]
    pub count_oligos_segment_index: Option<SegmentIndexOrAll>,

    /// Strategy for base and quality counting
    /// - 'original': Separate passes for base and quality counting (default)
    /// - 'optimized': Optimized quality counting with lookup tables
    /// - 'merged': Single-pass counting for both bases and qualities (fastest)
    #[serde(default)]
    pub counting_strategy: CountingStrategy,
    /// Generate histograms for specified tags
    #[serde(default)]
    pub tag_histograms: Option<Vec<String>>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            name: "report".to_string(),
            count: true,
            base_statistics: false,
            length_distribution: false,
            duplicate_count_per_read: false,
            duplicate_count_per_fragment: false,
            debug_reproducibility: false,
            count_oligos: None,
            count_oligos_segment: default_segment_all(),
            count_oligos_segment_index: None,
            tag_histograms: None,
            counting_strategy: CountingStrategy::default(),
        }
    }
}

impl Step for Report {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        let mut seen = HashSet::new();
        for t in all_transforms
            .iter()
            .filter(|t| matches!(t, Transformation::Report(_)))
        {
            match t {
                Transformation::Report(c) => {
                    if !seen.insert(c.name.clone()) {
                        bail!(
                            "Report labels must be distinct. Duplicated: \"{}\"",
                            self.name
                        )
                    }
                    if let Some(count_oligos) = c.count_oligos.as_ref() {
                        for oligo in count_oligos {
                            if oligo.is_empty() {
                                bail!("Oligo cannot be empty")
                            }
                            validate_dna(oligo.as_bytes())
                                .with_context(|| format!("validating oligo '{oligo}'"))?;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.count_oligos_segment_index = Some(self.count_oligos_segment.validate(input_def)?);
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
        panic!("Should not be reached - should be expanded into individual parts before");
    }

    fn apply(
        &mut self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        panic!("Should not be reached - should be expanded into individual parts before");
    }
}
