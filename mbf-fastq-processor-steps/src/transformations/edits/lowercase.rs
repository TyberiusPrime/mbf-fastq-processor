use crate::transformations::prelude::*;

/// Convert a read, name, tag into lower case
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Lowercase {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "source")]
    #[tpd(alias = "segment")]
    pub target: ResolvedSourceAll,

    pub if_tag: Option<ConditionalTagLabel>,
}

impl VerifyIn<PartialConfig> for PartialLowercase {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialLowercase> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        unreachable!("Should have been transformed before");
    }
}

impl Step for Lowercase {
    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!();
    }
}
