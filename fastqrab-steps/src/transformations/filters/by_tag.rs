use crate::transformations::prelude::*;

/// Filter reads by presence/value of a (non-numeric) tag

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct ByTag {
    in_label: TagLabel,
    keep_or_remove: super::super::KeepOrRemove,
}

impl TagUser for PartialTaggedVariant<PartialByTag> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(&[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                ])],
                must_see_all_tags: true, // for filtering them down
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl Step for ByTag {
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
