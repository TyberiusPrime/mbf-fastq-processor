#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::{Result, bail};
use bstr::BString;
use std::collections::BTreeMap;
use std::path::Path;

use super::{InputInfo, Step, TagValueType, Transformation};
use crate::demultiplex::{DemultiplexInfo, Demultiplex as CrateDemultiplex};
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct Demultiplex {
    pub label: String,
    pub output_unmatched: bool,
    // reference to shared barcodes section
    pub barcodes: String,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub resolved_barcodes: Option<BTreeMap<BString, String>>,
}

impl Step for Demultiplex {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate that either inline barcodes or reference to barcodes section is provided

        // Check only one demultiplex step
        let demultiplex_count = all_transforms
            .iter()
            .filter(|t| matches!(t, Transformation::Demultiplex(_)))
            .count();
        if demultiplex_count > 1 {
            bail!("Only one level of demultiplexing is supported.");
        }

        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.label.clone(), &[TagValueType::Location])])
    }

    fn resolve_config_references(
        &mut self,
        barcodes_data: &std::collections::HashMap<String, crate::config::Barcodes>,
    ) -> Result<()> {
        if let Some(barcodes_ref) = barcodes_data.get(&self.barcodes) {
            self.resolved_barcodes = Some(barcodes_ref.barcode_to_name.clone());
        } else {
            bail!(
                "Could not find referenced barcode section: {}",
                self.barcodes
            );
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &CrateDemultiplex,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexInfo>> {
        Ok(Some(DemultiplexInfo::new(
            self.resolved_barcodes.as_ref().unwrap(),
            self.output_unmatched,
        )?))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        demultiplex_info: &CrateDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let hits = block
            .tags
            .as_ref()
            .expect("No hits? bug")
            .get(&self.label)
            .expect("Label not present. Should have been caught in validation");
        let mut tags: Vec<u16> = vec![0; block.len()];
        let demultiplex_info = demultiplex_info.demultiplexed.unwrap();
        for (ii, target_tag) in tags.iter_mut().enumerate() {
            let key = hits[ii]
                .as_sequence()
                .map(|x| x.joined_sequence(Some(b"_")))
                .unwrap_or_default();
            let entry = demultiplex_info.barcode_to_tag(&key);
            match entry {
                Some(tag) => {
                    *target_tag = tag;
                }
                None => {
                    // No exact match found - tag remains 0 (unmatched)
                }
            }
        }
        block.output_tags = Some(tags);
        Ok((block, true))
    }
}
