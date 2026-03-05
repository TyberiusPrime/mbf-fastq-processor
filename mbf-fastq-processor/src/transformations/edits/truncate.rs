#![allow(clippy::unnecessary_wraps)] //eserde false positives

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
        //todo: reenable this check, fix tests
        // self.n.verify(|v| {
        //     if *v == 0 {
        //         Err(ValidationFailure::new(
        //             "n must be > 0",
        //             Some("Set to a positive integer."),
        //         ))
        //     } else {
        //         Ok(())
        //     }
        // });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialTruncate> {
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
            used_tags: vec![inner.if_tag.to_used_tag(&[][..])],
            must_see_all_tags: true,
            ..Default::default()
        }
    }
}

impl Step for Truncate {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self
            .if_tag
            .as_ref()
            .map(|tag| get_bool_vec_from_tag(&block, tag));

        block.apply_in_place(
            self.segment,
            |read| read.max_len(self.n),
            condition.as_deref(),
        );
        block.filter_tag_locations_beyond_read_length(self.segment);
        Ok((block, true))
    }
}
