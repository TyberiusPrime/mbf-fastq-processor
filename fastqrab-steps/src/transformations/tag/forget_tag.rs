use crate::transformations::prelude::*;

/// remove one tag from memory

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct ForgetTag {
    in_label: TagLabel,
}

impl TagUser for PartialTaggedVariant<PartialForgetTag> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                removed_tags: RemovedTags::Some(vec![(
                    inner.in_label.as_ref().expect("parent was ok").clone(),
                    &mut inner.in_label,
                )]),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ForgetTag {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.tags.shift_remove(&self.in_label);
        Ok((block, true))
    }
}
