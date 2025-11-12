#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;
use anyhow::{Result, bail};
use bstr::BString;
use std::collections::BTreeMap;
use std::path::Path;

use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Demultiplex {
    pub in_label: String,
    #[serde(default)]
    pub output_unmatched: Option<bool>,
    // reference to shared barcodes section (optional for boolean tag mode)
    #[serde(default)]
    pub barcodes: Option<String>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub resolved_barcodes: Option<BTreeMap<BString, String>>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    any_hit_observed: bool,
}

impl Step for Demultiplex {
    fn needs_serial(&self) -> bool {
        true
    }

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
                if tag_label == self.in_label {
                    upstream_label_type = Some(tag_type);
                    break;
                }
            }
        }
        if upstream_label_type.is_none() {
            bail!("Upstream label {} not found", self.in_label);
        }
        let upstream_label_is_bool = matches!(upstream_label_type, Some(TagValueType::Bool));
        if !upstream_label_is_bool {
            if self.output_unmatched.is_none() {
                bail!("output_unmatched must be set when using barcodes for demultiplex");
            }
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(
            self.in_label.clone(),
            &[
                TagValueType::Location,
                TagValueType::String,
                TagValueType::Bool,
            ],
        )])
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        assert!(!self.any_hit_observed);

        let barcodes_data = &input_info.barcodes_data;
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
                format!("{label}=false", label = self.in_label),
            );
            synthetic_barcodes.insert(
                BString::from("true"),
                format!("{label}=true", label = self.in_label),
            );
            self.resolved_barcodes = Some(synthetic_barcodes);
            self.output_unmatched = Some(false);
        }

        Ok(Some(DemultiplexBarcodes {
            barcode_to_name: self.resolved_barcodes.as_ref().unwrap().clone(),
            include_no_barcode: self.output_unmatched.unwrap(),
        }))
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let hits = block
            .tags
            .get(&self.in_label)
            .expect("Label not present. Should have been caught in validation");
        let demultiplex_info = demultiplex_info.unwrap();

        let mut output_tags = block
            .output_tags
            .take()
            .unwrap_or_else(|| vec![0; block.len()]);

        for (ii, tag_value) in hits.iter().enumerate() {
            let key: BString = match tag_value {
                crate::dna::TagValue::Location(hits) => hits.joined_sequence(Some(b"_")).into(),
                crate::dna::TagValue::String(bstring) => bstring.clone(),
                crate::dna::TagValue::Bool(bool_val) => {
                    if *bool_val {
                        b"true".into()
                    } else {
                        b"false".into()
                    }
                }
                crate::dna::TagValue::Missing => {
                    continue;
                } // leave at 0.
                _ => {
                    unreachable!();
                }
            };
            if let Some(tag) = demultiplex_info.barcode_to_tag(&key) {
                output_tags[ii] |= tag;
                if tag > 0 {
                    self.any_hit_observed = true;
                }
            }
        }

        block.output_tags = Some(output_tags);
        Ok((block, true))
    }

    fn finalize(
        &mut self,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        if !self.any_hit_observed {
            bail!(
                "Demultiplex step for label '{}' did not observe any matching barcodes. Please check that the barcodes section matches the data, or that the correct tag label is used.",
                self.in_label
            );
        }
        Ok(None)
    }
}
