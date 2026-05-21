use toml_pretty_deser::PartialTaggedVariant;

use super::extract_numeric_tags_plus_all;
use crate::transformations::prelude::*;

/// Convert read length into a tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Length {
    pub out_label: TagLabel,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialLength {
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

impl TagUser for PartialTaggedVariant<PartialLength> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric((
                    Some(NonNaN::new(0.0).expect("can't fail")),
                    None,
                ))),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Length {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment,
            &self.out_label,
            #[expect(
                clippy::cast_precision_loss,
                reason = "loss is acceptable, it's going to be within u32 range"
            )]
            |read| read.seq().len() as f64,
            #[expect(
                clippy::cast_precision_loss,
                reason = "loss is acceptable, it's going to be within u32 range"
            )]
            |reads| {
                let total_length: usize = reads.iter().map(|read| read.seq().len()).sum();
                total_length as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}
