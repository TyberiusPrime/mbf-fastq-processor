#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::super::KeepOrRemove;

/// Filter reads by threshold on a (numeric) tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ByNumericTag {
    pub in_label: String,
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
        self.in_label.verify(|v| {
            if v.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
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

impl Step for ByNumericTag {
    fn must_see_all_tags(&self) -> bool {
        true
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Numeric])])
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let tag_values = block
            .tags
            .get(&self.in_label)
            .expect("Numeric tag not found");

        let keep: Vec<bool> = tag_values
            .iter()
            .map(|tag_val| {
                if let Some(value) = tag_val.as_numeric() {
                    let passes_min = self.min_value.is_none_or(|min| value >= min);
                    let passes_max = self.max_value.is_none_or(|max| value < max);
                    passes_min && passes_max
                } else {
                    false // Non-numeric values are filtered out
                }
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
