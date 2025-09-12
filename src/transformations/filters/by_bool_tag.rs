use anyhow::Result;

use super::super::{KeepOrRemove, Step, TagValueType, Transformation};
use crate::demultiplex::Demultiplexed;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByBoolTag {
    pub label: String,
    pub keep_or_remove: KeepOrRemove,
}

impl Step for ByBoolTag {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        // Check that the required tag is declared as Bool by an upstream step
        let mut found_tag_declaration = false;
        for (i, transform) in all_transforms.iter().enumerate() {
            if i >= this_transforms_index {
                break; // Only check upstream steps
            }
            if let Some((tag_name, tag_type)) = transform.declares_tag_type() {
                if tag_name == self.label {
                    found_tag_declaration = true;
                    if tag_type != TagValueType::Bool {
                        return Err(anyhow::anyhow!(
                            "FilterByBoolTag step expects bool tag '{}', but upstream step declares {:?} tag",
                            self.label,
                            tag_type
                        ));
                    }
                    break;
                }
            }
        }

        if !found_tag_declaration {
            return Err(anyhow::anyhow!(
                "FilterByBoolTag step expects bool tag '{}', but no upstream step declares this tag",
                self.label
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
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let tag_values = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Bool tag not found");

        let keep: Vec<bool> = tag_values
            .iter()
            .map(|tag_val| {
                if let Some(passes) = tag_val.as_bool() {
                    passes
                } else {
                    // This should not happen due to validation, but handle gracefully
                    false
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
