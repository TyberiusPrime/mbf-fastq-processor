#![allow(clippy::unnecessary_wraps)] use indexmap::IndexMap;

//eserde false positives
use crate::transformations::prelude::*;
use std::collections::BTreeMap;

///Create multiple output files based on a tag
#[derive(eserde::Deserialize, Debug, JsonSchema)]
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
    pub resolved_barcodes: Option<IndexMap<BString, String>>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    any_hit_observed: std::sync::atomic::AtomicBool,
}

impl Step for Demultiplex {
    // fn needs_serial(&self) -> bool {
    //     true
    // }

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
            if let Some((tag_label, tag_type)) = trafo.declares_tag_type()
                && tag_label == self.in_label
            {
                upstream_label_type = Some(tag_type);
                break;
            }
        }
        let upstream_label_is_bool = matches!(upstream_label_type, Some(TagValueType::Bool));
        if !upstream_label_is_bool && self.output_unmatched.is_none() {
            bail!(
                "output_unmatched must be set when using barcodes for demultiplex. Set it to true or false as needed."
            );
        }
        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
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
        assert!(
            !self
                .any_hit_observed
                .load(std::sync::atomic::Ordering::Relaxed)
        );

        let barcodes_data = &input_info.barcodes_data;
        if let Some(barcodes_name) = &self.barcodes {
            // Barcode mode - resolve barcode reference
            if let Some(barcodes_ref) = barcodes_data.get(barcodes_name) {
                self.resolved_barcodes = Some(barcodes_ref.barcode_to_name.clone());
            } else {
                bail!("Could not find referenced barcode section: {barcodes_name}",);
            }
        } else {
            // Boolean tag mode - create synthetic barcodes for true/false
            let mut synthetic_barcodes = IndexMap::new();
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
            barcode_to_name: self
                .resolved_barcodes
                .as_ref()
                .expect("resolved_barcodes must be set during initialization")
                .clone(),
            include_no_barcode: self
                .output_unmatched
                .expect("output_unmatched must be set during initialization"),
        }))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let hits = block
            .tags
            .get(&self.in_label)
            .expect("Label not present. Should have been set in used_tags.");
        let demultiplex_info =
            demultiplex_info.expect("demultiplex_info must be Some in this code path");

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
                crate::dna::TagValue::Numeric(_) => {
                    unreachable!();
                }
            };
            if let Some(tag) = demultiplex_info.barcode_to_tag(&key) {
                output_tags[ii] |= tag;
                if tag > 0 {
                    self.any_hit_observed
                        .store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
        }

        block.output_tags = Some(output_tags);
        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        if !self
            .any_hit_observed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            bail!(
                "Demultiplex step for label '{}' did not observe any matching barcodes. Please check that the barcodes section matches the data, or that the correct tag label is used.",
                self.in_label
            );
        }
        Ok(None)
    }
}