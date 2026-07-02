use crate::transformations::prelude::*;

/// Remove all tags from memory

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
#[expect(dead_code, reason = "TDP needs at least one field")]
pub struct ForgetAllTags {
    #[schemars(skip)]
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
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Removal is hoisted into the pipeline: the stage declares
        // `RemovedTags::All`, and the workpool drops every tag before this
        // `apply` runs. Nothing left to do here.
        Ok((block, true))
    }
}
