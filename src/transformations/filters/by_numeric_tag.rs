#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;

use super::super::{KeepOrRemove, Step, Transformation};
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByNumericTag {
    pub label: String,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub keep_or_remove: KeepOrRemove,
}

impl Step for ByNumericTag {
    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.min_value.is_none() && self.max_value.is_none() {
            return Err(anyhow::anyhow!(
                "At least one of min_value or max_value must be specified"
            ));
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        Some(vec![self.label.clone()])
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let tag_values = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
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

        super::super::apply_bool_filter(&mut block, &keep);
        (block, true)
    }
}
