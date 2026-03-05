#![allow(clippy::unnecessary_wraps)]
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
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        TagUsageInfo {
            removed_tags: RemovedTags::Some(vec![(
                inner.in_label.as_ref().expect("parent was ok").clone(),
                &mut inner.in_label,
            )]),
            ..Default::default()
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
        block.tags.remove(&self.in_label);
        Ok((block, true))
    }
}
