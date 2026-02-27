#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

/// Filter empty reads (without sequence, length == 0)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Empty {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialEmpty {
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

impl Step for Empty {
    fn apply(
        &self,
        mut _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!("Should have been replaced before validation");
    }
}
