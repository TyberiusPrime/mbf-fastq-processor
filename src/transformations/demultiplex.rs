#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::{bail, Result};
use bstr::BString;
use noodles::sam::header::record::value::map::Tag;
use std::collections::BTreeMap;
use std::path::Path;

use super::{InputInfo, Step, TagValueType, Transformation};
use crate::demultiplex::{Demultiplex as CrateDemultiplex, DemultiplexInfo};
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct Demultiplex {
    pub label: String,
    pub output_unmatched: bool,
    // reference to shared barcodes section (optional for boolean tag mode)
    #[serde(default)]
    pub barcodes: Option<String>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub resolved_barcodes: Option<BTreeMap<BString, String>>,

    #[serde(default)]
    #[serde(skip)]
    pub is_bool_tag: bool,
}

impl Step for Demultiplex {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        // Multiple demultiplex steps are now supported
        // Each demultiplex step defines a bit region for its variants
        // When demultiplexing, they are combined with OR logic
        let upstream_label_is_bool: bool = (|| {
            for trafo in all_transforms[..this_transforms_index - 1].iter().rev() {
                if let Some((tag_label, tag_type)) = trafo.declares_tag_type() {
                    if tag_label == self.label {
                        return match tag_type {
                            TagValueType::Bool => true,
                            TagValueType::String | TagValueType::Location => false,
                            _ => unreachable!(),
                        };
                    }
                }
            }
            false
        })();
        if !upstream_label_is_bool && self.barcodes.is_none() {
            bail!("Demultiplex step using tag label '{}' must reference a barcodes section (exception: bool tags, but {} isn't a bool tag)", self.label, self.label);
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(
            self.label.clone(),
            &[
                TagValueType::Location,
                TagValueType::String,
                TagValueType::Bool,
            ],
        )])
    }

    fn resolve_config_references(
        &mut self,
        barcodes_data: &std::collections::HashMap<String, crate::config::Barcodes>,
    ) -> Result<()> {
        if let Some(barcodes_name) = &self.barcodes {
            // Barcode mode - resolve barcode reference
            if let Some(barcodes_ref) = barcodes_data.get(barcodes_name) {
                self.resolved_barcodes = Some(barcodes_ref.barcode_to_name.clone());
                self.is_bool_tag = false;
            } else {
                bail!(
                    "Could not find referenced barcode section: {}",
                    barcodes_name
                );
            }
        } else {
            // Boolean tag mode - create synthetic barcodes for true/false
            let mut synthetic_barcodes = BTreeMap::new();
            synthetic_barcodes.insert(
                BString::from("false"),
                format!("{label}=false", label = self.label),
            );
            synthetic_barcodes.insert(
                BString::from("true"),
                format!("{label}=true", label = self.label),
            );
            self.resolved_barcodes = Some(synthetic_barcodes);
            self.is_bool_tag = true;
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
        let info = DemultiplexInfo::new(
            self.resolved_barcodes.as_ref().unwrap(),
            self.output_unmatched,
        )?;
        // Store our own demultiplex info for later use in apply()
        Ok(Some(info))
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

        // Use our own stored demultiplex info, not the combined one
        let my_info = &demultiplex_info.demultiplexed.unwrap();
        let my_output_count = my_info.output_count();

        // Check if there are existing output tags from a previous demultiplex
        let existing_tags = block.output_tags.take();
        let mut new_tags: Vec<u16> = vec![0; block.len()];

        if self.is_bool_tag {
            // Boolean tag mode - convert bool values to strings
            for (ii, target_tag) in new_tags.iter_mut().enumerate() {
                if let Some(bool_val) = hits[ii].as_bool() {
                    let key = if bool_val {
                        BString::from("true")
                    } else {
                        BString::from("false")
                    };
                    let entry = my_info.barcode_to_tag(&key);
                    match entry {
                        Some(tag) => {
                            *target_tag = tag;
                        }
                        None => {
                            // No exact match found - tag remains 0 (unmatched)
                        }
                    }
                } else {
                    // Missing tag value - treat as unmatched (tag 0)
                }
            }
        } else {
            // Barcode mode - use sequence values
            for (ii, target_tag) in new_tags.iter_mut().enumerate() {
                let key = hits[ii]
                    .as_sequence()
                    .map(|x| x.joined_sequence(Some(b"_")))
                    .unwrap_or_default();
                let entry = my_info.barcode_to_tag(&key);
                match entry {
                    Some(tag) => {
                        *target_tag = tag;
                    }
                    None => {
                        // No exact match found - tag remains 0 (unmatched)
                    }
                }
            }
        }

        // Combine with existing tags if present (for multiple demultiplex steps)
        if let Some(existing) = existing_tags {
            for (ii, new_tag) in new_tags.iter_mut().enumerate() {
                let old_tag = existing[ii];
                // Combined tag = old_tag * my_output_count + new_tag
                *new_tag = old_tag * (my_output_count as u16) + *new_tag;
            }
        }

        block.output_tags = Some(new_tags);
        Ok((block, true))
    }
}
