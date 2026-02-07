#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

/// Filter empty reads (without sequence, length == 0)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Empty {
    #[tpd_default]
    pub segment: SegmentOrAll,
    #[tpd_skip]
    #[schemars(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
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

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }
}
