use crate::transformations::prelude::*;

/// remove one tag from memory

#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct ForgetTag {
    // Read at config-verify (on the partial, to declare `removed_tags`); the
    // runtime removal is hoisted into the workpool, so `apply` never reads it.
    #[expect(dead_code, reason = "consumed at config-verify, not at runtime")]
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
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // The actual removal of `in_label` is hoisted into the pipeline: the
        // stage declares it via `removed_tags`, and the workpool drops it before
        // this `apply` runs. Nothing left to do here.
        Ok((block, true))
    }
}
