#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

/// Convert a read, name, tag into upper case
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Uppercase {
    #[tpd(alias = "segment")]
    #[tpd(alias = "source")]
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub target: ResolvedSourceAll,

    pub if_tag: Option<TagLabel>,
}

impl VerifyIn<PartialConfig> for PartialUppercase {
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

impl TagUser for PartialTaggedVariant<PartialUppercase> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        unreachable!("Should have been transformed before");
    }
}

impl Step for Uppercase {
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        Ok((block, true))
    }
}
