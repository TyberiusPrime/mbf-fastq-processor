//#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

/// Count the number of N. See `CalcBaseContent` for general case
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct NCount {
    pub out_label: TagLabel,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialNCount {
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

impl TagUser for PartialTaggedVariant<PartialNCount> {
    fn get_tag_usage(&mut self,_tags_available: &IndexMap<TagLabel,TagMetadata>,_segment_order: &[String],) -> TagUsageInfo<'_>{
        unreachable!("Should have been swapped for BaseCount in expansion");
    }
}

impl Step for NCount {

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        bail!("ExtractNCount is converted into ExtractBaseContent during expansion")
    }
}
