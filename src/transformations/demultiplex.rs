#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::{bail, Result};
use bstr::BString;
use std::collections::BTreeMap;
use std::path::Path;

use super::{InputInfo, Step, TagValueType, Transformation};
use crate::demultiplex::{
    self, Demultiplex as CrateDemultiplex, DemultiplexBarcodes, DemultiplexInfo,
};
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
        let mut upstream_label_type = None;
        for trafo in all_transforms[..this_transforms_index].iter().rev() {
            if let Some((tag_label, tag_type)) = trafo.declares_tag_type() {
                if tag_label == self.label {
                    upstream_label_type = Some(tag_type);
                    break;
                }
            }
        }
        if upstream_label_type.is_none() {
            bail!("Upstream label {} not found", self.label);
        }
        let upstream_label_is_bool = matches!(upstream_label_type, Some(TagValueType::Bool));
        if self.barcodes.is_none() && !upstream_label_is_bool {
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
            self.output_unmatched = false;
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: Option<&DemultiplexInfo>,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        Ok(Some(DemultiplexBarcodes {
            barcode_to_name: self.resolved_barcodes.as_ref().unwrap().clone(),
            include_no_barcode: self.output_unmatched,
        }))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        demultiplex_info: Option<&DemultiplexInfo>,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let hits = block
            .tags
            .as_ref()
            .expect("No hits? bug")
            .get(&self.label)
            .expect("Label not present. Should have been caught in validation");
        let demultiplex_info = demultiplex_info.unwrap();

        let mut output_tags = block
            .output_tags
            .take()
            .unwrap_or_else(|| vec![0; block.len()]);

        for (ii, tag_value) in hits.iter().enumerate() {
            let key = match tag_value {
                crate::dna::TagValue::Location(hits) => hits.joined_sequence(Some(b"_")),
                crate::dna::TagValue::String(bstring) => bstring.to_vec(),
                crate::dna::TagValue::Bool(bool_val) => {
                    if *bool_val {
                        b"true".to_vec()
                    } else {
                        b"false".to_vec()
                    }
                }
                crate::dna::TagValue::Missing => {
                    continue;
                } // leave at 0.
                _ => {
                    dbg!(&hits[ii]);
                    unreachable!();
                }
            };
            if let Some(tag) = demultiplex_info.barcode_to_tag.get(&key) {
                output_tags[ii] |= tag;
            }
        }

        block.output_tags = Some(output_tags);
        Ok((block, true))
    }
}
