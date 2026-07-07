use crate::transformations::prelude::*;

/// Truncate reads to a fixed length
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Truncate {
    n: usize,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,
    #[tpd(alias = "if_label")]
    if_tag: Option<ConditionalTagLabel>,
}

impl VerifyIn<PartialConfig> for PartialTruncate {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialTruncate> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.if_tag.to_used_tag(&[])],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Truncate {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self
            .if_tag
            .as_ref()
            .map(|tag| get_bool_vec_from_tag(&block, tag));

        block
            .member_mut(self.segment.as_index())
            .seq_quals
            .max_len(self.n, condition.as_deref());
        Ok((block, true))
    }
}
