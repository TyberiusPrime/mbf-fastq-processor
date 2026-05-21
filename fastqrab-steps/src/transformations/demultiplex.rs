use indexmap::IndexMap;

use crate::transformations::prelude::*;

///Create multiple output files based on a tag

#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Demultiplex {
    pub in_label: TagLabel,
    pub output_unmatched: bool,

    /// reference to shared barcodes section (optional for boolean tag mode)
    pub barcodes: Option<TagLabel>,

    // by default, set from tag type...
    pub tag_contains_barcode: Option<bool>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub resolved_barcodes: IndexMap<BString, String>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    any_hit_observed: std::sync::atomic::AtomicBool,

    #[tpd(skip)]
    #[schemars(skip)]
    pub lookup_mode: LookupMode,
}

#[derive(Debug)]
pub enum LookupMode {
    NoLookup,
    Lookup,
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
        let ix_separator = parent
            .output
            .as_ref()
            .and_then(|x| x.as_ref().and_then(|x| x.ix_separator.as_ref()));
        if let Some(Some(barcodes_name)) = self.barcodes.as_ref() {
            if let Some(Some(barcodes)) = parent.barcodes.value.as_ref() {
                //error sections are
                //ok...
                if let Some(barcodes_ref) = barcodes.map.get(barcodes_name.as_ref()) {
                    if let Some(resolved) = barcodes_ref
                        .as_ref()
                        .and_then(|x| x.seq_to_name.as_ref())
                        .map(|x| {
                            x.iter()
                                .map(|(k, v)| (k.clone(), v.clone()))
                                .collect::<IndexMap<_, _>>()
                        })
                    {
                        for v in resolved.values() {
                            if v.contains("..")
                                || v.contains('/')
                                || v.contains('\\')
                                || v.contains(':')
                            {
                                return Err(ValidationFailure {
                                    message: "Invalid barcode name found".to_string(),
                                    help: Some(format!(
                                        "Barcode names that lead to filenames cannot contain '..', '/', ':' or '\\'\n\
                                        Found: '{v}'"
                                    )),
                                });
                            }
                            if let Some(ix_separator) = ix_separator
                                && v.contains(ix_separator)
                            {
                                return Err(ValidationFailure {
                                    message: "Invalid barcode name found".to_string(),
                                    help: Some(format!(
                                        "Barcode names must not contain the output.ix_separator '{ix_separator}'\n\
                                        Barcode name in question: '{v}'\n\
                                        Change the output.ix_separator in your config or remove it from the barcode name."
                                    )),
                                });
                            }
                        }

                        self.resolved_barcodes = Some(resolved);
                    } else {
                        //not a valid barcode, error message will have been generated elsewhere.
                        self.resolved_barcodes = None;
                    }
                } else {
                    self.barcodes.help = Some(offer_alternatives(
                        barcodes_name.as_ref(),
                        &barcodes
                            .keys
                            .iter()
                            .filter_map(|x| x.as_ref())
                            .collect::<Vec<_>>(),
                    ));
                    self.barcodes.state = TomlValueState::new_validation_failed(
                        "Unknown barcode section".to_string(),
                    );
                    return Ok(());
                }
            } else {
                self.barcodes.help = Some(
                    "There is no valid [barcodes.<barcodes_name>] section in your TOML. Add one."
                        .to_string(),
                );
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
                    format!("{label}=false", label = label.as_ref()),
                );
                synthetic_barcodes.insert(
                    BString::from("true"),
                    format!("{label}=true", label = label.as_ref()),
                );
                self.resolved_barcodes = Some(synthetic_barcodes);
            }
            self.output_unmatched.or(false); // unused for bool.
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialDemultiplex> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.as_ref() {
            // Multiple demultiplex steps are now supported
            // Each demultiplex step defines a bit region for its variants
            // When demultiplexing, they are combined with OR logic
            let upstream_label_type = tags_available
                .get(inner.in_label.as_ref().expect("parent was ok"))
                .map(|meta| &meta.tag_type);
            let inner = self
                .toml_value
                .value
                .as_mut()
                .expect("Was ok before, now might not be ok, but should be still set");

            if let Some(upstream_label_type) = upstream_label_type {
                match upstream_label_type {
                    TagValueType::Location | TagValueType::String => {
                        if let Some(Some(tag_contains_barcode)) =
                            inner.tag_contains_barcode.as_ref()
                        {
                            //user has explicitly told us what the tag contains,
                            //barcodes or barcode-names
                            if *tag_contains_barcode {
                                inner.lookup_mode = Some(LookupMode::Lookup);
                            } else {
                                inner.lookup_mode = Some(LookupMode::NoLookup);
                            }
                        } else if matches!(upstream_label_type, TagValueType::Location) {
                            inner.lookup_mode = Some(LookupMode::Lookup);
                        } else {
                            inner.lookup_mode = Some(LookupMode::NoLookup);
                        }
                    }
                    // cov:excl-start
                    TagValueType::Numeric(_) => {
                        //will be complained about because of allowed tag modes below
                    }
                    // cov:excl-stop
                    TagValueType::Bool => {
                        // if inner.output_unmatched.as_ref().is_some() {
                        //     self.toml_value.state = TomlValueState::new_validation_failed(
                        //         "output_unmatched must be *not* set when using boolean values for demultiplex.",
                        //     );
                        //     self.toml_value.help =
                        //         Some("Remove output_unmatched=true (or false)".to_string());
                        // }
                        inner.lookup_mode = Some(LookupMode::Lookup);
                        inner.output_unmatched.value = Some(false);
                    }
                }
            }

            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(
                    &[
                        TagValueType::Bool,
                        TagValueType::String,
                        TagValueType::Location,
                    ][..],
                )],
                used_barcodes: inner
                    .barcodes
                    .as_ref()
                    .and_then(Option::as_ref)
                    .cloned()
                    .into_iter()
                    .collect(),
                ..Default::default()
            })
        } else {
            None
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
        _output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        assert!(
            !self
                .any_hit_observed
                .load(std::sync::atomic::Ordering::Relaxed)
        );

        Ok(Some(DemultiplexBarcodes {
            barcode_to_name: self.resolved_barcodes.clone(),
            include_no_barcode: self.output_unmatched,
        }))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
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

        for (ii, key) in hits.iter_stringified().enumerate() {
            if let Some(key) = key {
                match self.lookup_mode {
                    LookupMode::Lookup => {
                        if let Some(tag) = demultiplex_info.barcode_to_tag(&key) {
                            output_tags[ii] |= tag;
                            if tag > 0 {
                                self.any_hit_observed
                                    .store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                    LookupMode::NoLookup => {
                        if let Some(tag) = demultiplex_info
                        .name_to_tag(std::str::from_utf8(&key).expect(
                        "Tag sequence was not utf-8, barcode names must be utf-8 unicode strings",
                    )) {
                        output_tags[ii] |= tag;
                        if tag > 0 {
                            self.any_hit_observed
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    } // cov:excl-line
                    }
                }
            } //else =missing, leave output tag at 0, which means unmatched
        }

        block.output_tags = Some(output_tags);
        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        if !self
            .any_hit_observed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let mut msg = format!(
                "Demultiplex step for label '{}' did not observe any matching barcodes.\n\
                    Please check that the barcodes section matches the data,\n\
                    or that the correct tag label is used.",
                self.in_label
            );
            if matches!(self.lookup_mode, LookupMode::NoLookup)
                && self.tag_contains_barcode.is_none()
            {
                msg.push_str("\nYou might need to set tag_contains_barcode=true to trigger the lookup barcode->sequence.");
            }
            bail!(msg);
        }
        Ok(None)
    }
}
