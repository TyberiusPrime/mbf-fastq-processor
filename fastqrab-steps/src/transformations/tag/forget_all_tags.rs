use crate::transformations::prelude::*;

/// Remove all tags from memory

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
#[expect(dead_code, reason ="TDP needs at least one field")]
pub struct ForgetAllTags {
    ignored: Option<u8>, //tdp dislikes empty structs
}

impl TagUser for PartialTaggedVariant<PartialForgetAllTags> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        Some(TagUsageInfo {
            removed_tags: RemovedTags::All,
            ..Default::default()
        })
    }
}

impl Step for ForgetAllTags {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        block.tags = Default::default();
        Ok((block, true))
    }
}
