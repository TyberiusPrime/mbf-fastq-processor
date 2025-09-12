#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::Demultiplexed;

use super::super::{Step, Transformation};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ByTag {
    label: String,
    keep_or_remove: super::super::KeepOrRemove,
}

impl Step for ByTag {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        // Check that the required tag is declared by some upstream step
        let mut found_tag_declaration = false;
        for (i, transform) in all_transforms.iter().enumerate() {
            if i >= this_transforms_index {
                break; // Only check upstream steps
            }
            if let Some((tag_name, _tag_type)) = transform.declares_tag_type() {
                if tag_name == self.label {
                    found_tag_declaration = true;
                    break; // FilterByTag accepts any tag type
                }
            }
        }

        if !found_tag_declaration {
            return Err(anyhow::anyhow!(
                "FilterByTag step expects tag '{}', but no upstream step declares this tag",
                self.label
            ));
        }

        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let mut keep: Vec<bool> = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tag_val| !tag_val.is_missing())
            .collect();
        if self.keep_or_remove == super::super::KeepOrRemove::Remove {
            keep.iter_mut().for_each(|x| *x = !*x);
        }
        super::super::apply_bool_filter(&mut block, &keep);

        (block, true)
    }
}
