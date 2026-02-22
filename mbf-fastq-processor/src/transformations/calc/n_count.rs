//#![allow(clippy::unnecessary_wraps)]
use super::BaseContent;
use crate::transformations::prelude::*;

/// Count the number of N. See `CalcBaseContent` for general case
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct NCount {
    pub out_label: String,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialNCount {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}
impl NCount {
    pub(crate) fn into_base_content(self) -> BaseContent {
        BaseContent::for_n_count(self.out_label, self.segment)
    }
}

impl Step for NCount {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

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
