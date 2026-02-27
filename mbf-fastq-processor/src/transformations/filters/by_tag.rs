#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{dna::TagValue, transformations::prelude::*};

/// Filter reads by presence/value of a (non-numeric) tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ByTag {
    in_label: String,
    keep_or_remove: super::super::KeepOrRemove,
}

impl VerifyIn<PartialConfig> for PartialByTag {
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
        Ok(())
    }
}

impl Step for ByTag {
    fn must_see_all_tags(&self) -> bool {
        true
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
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
        block.apply_bool_filter(&keep);

        Ok((block, true))
    }
}
