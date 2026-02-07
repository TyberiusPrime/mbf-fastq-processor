use crate::transformations::prelude::*;

use super::super::validate_dna;
use super::common::default_true;
use std::collections::HashSet;

use super::super::tag::default_segment_all;

/// Include a report at this position
#[derive(Clone, JsonSchema)]
#[tpd(partial=false)]
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Report {
    pub name: String,
    #[tpd_default_in_verify]
    pub count: bool,
    #[tpd_default]
    pub base_statistics: bool,
    #[tpd_default]
    pub length_distribution: bool,
    #[tpd_default]
    pub duplicate_count_per_read: bool,
    #[tpd_default]
    pub duplicate_count_per_fragment: bool,

    #[schemars(skip)]
    #[tpd_default]
    pub debug_reproducibility: bool,

    pub count_oligos: Option<Vec<String>>,
    #[tpd_default_in_verify]
    pub count_oligos_segment: SegmentOrAll,

    /// Generate histograms for specified tags
    #[tpd_alias("tag_histogram")]
    pub tag_histograms: Option<Vec<String>>,
}

impl VerifyFromToml for PartialReport {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.name = self.name.verify(helper, |name: &String| {
            if name.is_empty() {
                Ok(())
            } else {
                Err(("Name must not be empty".to_string(), None))
            }
        });
        self.count = self.count.or_default(true);
        //TODO: count_oligos_segmen
        self
    }
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
            tag_histograms: None,
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

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        unreachable!("Should not be reached - should be expanded into individual parts before");
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        panic!("Should not be reached - should be expanded into individual parts before");
    }
}
