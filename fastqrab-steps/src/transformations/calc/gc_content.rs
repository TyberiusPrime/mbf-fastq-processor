use crate::transformations::prelude::*;

/// Calculate (relative) GC content. See `CalcBaseContent`
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct GCContent {
    pub out_label: TagLabel,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
    #[tpd(alias = "rate")]
    pub relative: bool,
}

impl VerifyIn<PartialConfig> for PartialGCContent {
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

// cov:excl-start
impl TagUser for PartialTaggedVariant<PartialGCContent> {
    #[mutants::skip]
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        unreachable!("Should have been swapped in expansion")
    }
}

impl Step for GCContent {
    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        bail!("ExtractGCContent is converted into ExtractBaseContent during expansion")
    }
}
// cov:excl-stop
