#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

use crate::config::{SegmentIndexOrAll, SegmentOrAll};

use super::BaseContent;

/// Count the number of N. See CalcBaseContent for general case
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NCount {
    pub out_label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
}

impl NCount {
    pub(crate) fn into_base_content(self) -> BaseContent {
        BaseContent::for_n_count(self.out_label, self.segment, self.segment_index)
    }
}

impl Step for NCount {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

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
