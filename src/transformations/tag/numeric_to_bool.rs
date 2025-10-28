use anyhow::Result;
use std::collections::HashMap;

use super::super::{Step, TagValueType};
use crate::{demultiplex::Demultiplexed, dna::TagValue};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NumericToBoolTag {
    pub source_label: String,
    pub target_label: String,
    #[serde(default)]
    pub min_value: Option<f64>,
    #[serde(default)]
    pub max_value: Option<f64>,
}

impl Step for NumericToBoolTag {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
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
        Some(vec![(self.source_label.clone(), TagValueType::Numeric)])
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.target_label.clone(), TagValueType::Bool))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let tag_values = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.source_label))
            .expect("Numeric tag not found");

        let keep: Vec<bool> = tag_values
            .iter()
            .map(|tag_val| {
                if let Some(value) = tag_val.as_numeric() {
                    let passes_min = self.min_value.is_none_or(|min| value >= min);
                    let passes_max = self.max_value.is_none_or(|max| value < max);
                    passes_min && passes_max
                } else {
                    false
                }
            })
            .collect();

        let bool_values: Vec<TagValue> = keep.into_iter().map(TagValue::Bool).collect();

        let tags = block.tags.get_or_insert_with(HashMap::new);
        tags.insert(self.target_label.clone(), bool_values);

        Ok((block, true))
    }
}
