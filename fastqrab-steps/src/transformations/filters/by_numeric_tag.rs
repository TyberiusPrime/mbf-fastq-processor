use super::super::KeepOrRemove;
use crate::transformations::prelude::*;

/// Filter reads by threshold on a (numeric) tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ByNumericTag {
    #[tpd(adapt_in_verify(String))]
    pub in_label: TagLabel,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub keep_or_remove: KeepOrRemove,
}

impl VerifyIn<PartialConfig> for PartialByNumericTag {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        //since options are not 'missing'
        if let Some(None) = self.min_value.value
            && let Some(None) = self.max_value.value
        {
            return Err(ValidationFailure::new(
                "At least one of min_value or max_value must be specified",
                None,
            ));
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialByNumericTag> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            inner
                .in_label
                .validate_incoming_tag_label(tags_available, segment_order);
            if let Some(in_label) = inner.in_label.as_ref().and_then(|x| x.as_ref_post())
                && let Some(TagValueType::Numeric((declared_lower, declared_upper))) =
                    tags_available.get(in_label).map(|x| x.tag_type)
            {
                let declared_lower: f64 = declared_lower.map_or(f64::NEG_INFINITY, Into::into);
                let declared_upper: f64 = declared_upper.map_or(f64::INFINITY, Into::into);
                if let Some(Some(lower_threshold)) = inner.min_value.as_ref() {
                    if *lower_threshold < declared_lower || *lower_threshold > declared_upper {
                        inner.min_value.state =
                            TomlValueState::new_validation_failed("Out of range");
                        inner.min_value.help = Some(format!(
                            "Supply a value between {declared_lower}..={declared_upper}"
                        ));
                    }
                } else if let Some(Some(upper_threshold)) = inner.max_value.as_ref()
                    && (*upper_threshold < declared_lower || *upper_threshold > declared_upper)
                {
                    inner.max_value.state = TomlValueState::new_validation_failed("Out of range");
                    inner.max_value.help = Some(format!(
                        "Supply a value between {declared_lower}..={declared_upper}"
                    ));
                }
            }

            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_label
                        .to_used_tag(&[TagValueType::Numeric((None, None))])
                        .map(|used_tag| {
                            used_tag.add_help(
                            "Either switch to FilterByTag, or change the tag you are filtering on.",
                        )
                        }),
                ],
                must_see_all_tags: true, // for filtering them down
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ByNumericTag {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let tag_values = block
            .tags
            .get(&self.in_label)
            .expect("Numeric tag not found");

        let keep: Vec<bool> = tag_values
            .iter_numeric()
            .map(|value| {
                let passes_min = self.min_value.is_none_or(|min| *value >= min);
                let passes_max = self.max_value.is_none_or(|max| *value < max);
                passes_min && passes_max
            })
            .map(|passes| {
                if self.keep_or_remove == KeepOrRemove::Remove {
                    !passes
                } else {
                    passes
                }
            })
            .collect();

        block.apply_bool_filter(&keep);
        Ok((block, true))
    }
}
