#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{dna::TagValue, transformations::prelude::*};

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ByTag {
    in_label: String,
    keep_or_remove: super::super::KeepOrRemove,
}

impl Step for ByTag {
    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
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

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut keep: Vec<bool> = block
            .tags
            .get(&self.in_label)
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(TagValue::truthy_val)
            .collect();
        if self.keep_or_remove == super::super::KeepOrRemove::Remove {
            for x in &mut keep {
                *x = !*x;
            }
        }
        super::super::apply_bool_filter(&mut block, &keep);

        Ok((block, true))
    }
}
