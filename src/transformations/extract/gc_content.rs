#![allow(clippy::unnecessary_wraps)]
// eserde false positives
use anyhow::{Result, bail};

use crate::{
    Demultiplexed,
    config::{SegmentIndexOrAll, SegmentOrAll},
};

use super::super::Step;
use super::BaseContent;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct GCContent {
    pub label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
}

impl GCContent {
    pub(crate) fn into_base_content(self) -> BaseContent {
        BaseContent::for_gc_replacement(self.label, self.segment, self.segment_index)
    }
}

impl Step for GCContent {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        bail!("ExtractGCContent is converted into ExtractBaseContent during expansion")
    }
}
