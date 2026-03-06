#![allow(clippy::unnecessary_wraps)]
use indexmap::IndexMap;
use toml_pretty_deser::suggest_alternatives;

//eserde false positives
use crate::transformations::prelude::*;

///Create multiple output files based on a tag

#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Demultiplex {
    pub in_label: TagLabel,
    pub output_unmatched: Option<bool>,

    /// reference to shared barcodes section (optional for boolean tag mode)
    pub barcodes: Option<TagLabel>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub resolved_barcodes: IndexMap<BString, String>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    any_hit_observed: std::sync::atomic::AtomicBool,
}

impl VerifyIn<PartialConfig> for PartialDemultiplex {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.in_label.verify(|v| {
            if v.0.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
        if let Some(Some(barcodes_name)) = self.barcodes.as_ref() {
            if let Some(Some(barcodes)) = parent.barcodes.value.as_ref() {
                //error sections are
                //ok...
                if let Some(barcodes_ref) = barcodes.map.get(barcodes_name.0.as_str()) {
                    if let Some(resolved) = barcodes_ref
                        .as_ref()
                        .and_then(|x| x.barcode_to_name.as_ref())
                        .map(|x| {
                            x.map
                                .iter()
                                .map(|(k, v)| {
                                    (k.clone(), v.as_ref().expect("parent was ok").clone())
                                })
                                .collect()
                        })
                    {
                        self.resolved_barcodes = Some(resolved);
                    } else {
                        //not a valid barcode, error message will have been generated elsewhere.
                        self.resolved_barcodes = None;
                    }
                } else {
                    dbg!(&barcodes.keys);
                    self.barcodes.help = Some(suggest_alternatives(
                        &barcodes_name.0,
                        &barcodes
                            .keys
                            .iter()
                            .filter_map(|x| x.as_ref())
                            .collect::<Vec<_>>(),
                    ));
                    self.barcodes.state =
                        TomlValueState::new_validation_failed("Unknown barcode section".to_string());
                    return Ok(());
                }
            } else {
                self.barcodes.help = Some("There is no valid [barcodes.<barcodes_name>] section in your TOML. Add one.".to_string());
                self.barcodes.state =
                    TomlValueState::new_validation_failed("Unknown barcode section".to_string());
                return Ok(());
            }
        } else {
            // Boolean tag mode - create synthetic barcodes for true/false
            let mut synthetic_barcodes = IndexMap::new();
            if let Some(label) = self.in_label.as_ref() {
                synthetic_barcodes.insert(
                    BString::from("false"),
                    format!("{label}=false", label = label.0),
                );
                synthetic_barcodes.insert(
                    BString::from("true"),
                    format!("{label}=true", label = label.0),
                );
                self.resolved_barcodes = Some(synthetic_barcodes);
                self.output_unmatched.value = Some(Some(false));
                self.output_unmatched.state = TomlValueState::Ok;
            }
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialDemultiplex> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_ref()
            .expect("get_tag_usage should only be called after successful verification");

        // Multiple demultiplex steps are now supported
        // Each demultiplex step defines a bit region for its variants
        // When demultiplexing, they are combined with OR logic
        let upstream_label_type = tags_available
            .get(inner.in_label.as_ref().expect("parent was ok"))
            .map(|meta| &meta.tag_type);
        let upstream_label_is_bool = matches!(upstream_label_type, Some(TagValueType::Bool));
        if !upstream_label_is_bool
            && inner
                .output_unmatched
                .as_ref()
                .expect("parent was ok")
                .is_none()
        {
            self.toml_value.state = TomlValueState::new_validation_failed(
                "output_unmatched must be set when using barcodes for demultiplex.",
            );
            self.toml_value.help = Some("Add output_unmatched=true (or false)".to_string());
        }
        let inner = self
            .toml_value
            .value
            .as_mut()
            .expect("Was ok before, now might not be ok, but should be still set");

        TagUsageInfo {
            used_tags: vec![inner.in_label.to_used_tag(
                &[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ][..],
            )],
            ..Default::default()
        }
    }
}

impl Step for Demultiplex {
    // fn needs_serial(&self) -> bool {
    //     true
    // }

    fn init(
        &mut self,
        _input_info: &InputInfo,
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

        Ok(Some(DemultiplexBarcodes {
            barcode_to_name: self.resolved_barcodes.clone(),
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
