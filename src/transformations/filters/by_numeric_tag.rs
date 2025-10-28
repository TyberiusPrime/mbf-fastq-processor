#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;

use super::super::{KeepOrRemove, Step, TagValueType, Transformation};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByNumericTag {
    pub label: String,
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
    pub keep_or_remove: KeepOrRemove,
}

impl Step for ByNumericTag {
    fn validate_others(
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

    fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
        Some(vec![(self.label.clone(), TagValueType::Numeric)])
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &crate::demultiplex::Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        unreachable!(
            "FilterByNumericTag is expected to expand into NumericToBoolTag + FilterByBoolTag"
        );
    }
}
