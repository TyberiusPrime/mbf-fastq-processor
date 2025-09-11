use super::super::{validate_dna, InputInfo, Step, Transformation};
use super::common::default_true;
use crate::config::{SegmentIndexOrAll, SegmentOrAll};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use anyhow::{bail, Context, Result};
use std::collections::HashSet;

use super::super::tag::default_segment_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct Report {
    pub label: String,
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
    pub debug_reproducibility: bool,

    pub count_oligos: Option<Vec<String>>,
    
    #[serde(default = "default_segment_all")]
    count_oligos_segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub count_oligos_segment_index: Option<SegmentIndexOrAll>,
}

impl Default for Report {
    fn default() -> Self {
        Self {
            label: "report".to_string(),
            count: true,
            base_statistics: false,
            length_distribution: false,
            duplicate_count_per_read: false,
            duplicate_count_per_fragment: false,
            debug_reproducibility: false,
            count_oligos: None,
            count_oligos_segment: default_segment_all(),
            count_oligos_segment_index: None,
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
                    if !seen.insert(c.label.clone()) {
                        bail!(
                            "Report labels must be distinct. Duplicated: \"{}\"",
                            self.label
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
        _output_directory: &std::path::Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        panic!("Should not be reached - should be expanded into individual parts before");
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        panic!("Should not be reached - should be expanded into individual parts before");
    }
}
