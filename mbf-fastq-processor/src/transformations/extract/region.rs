#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::{RegionAnchor, prelude::*};

/// Define a region by coordinates
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Region {
    pub start: isize,
    #[tpd_alias("length")]
    pub len: usize,

    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[tpd_alias("segment")]
    pub source: String,

    #[schemars(skip)]
    #[tpd_skip]
    pub resolved_source: Option<ResolvedSourceNoAll>,

    /// Is the region from the `Start` or the `End` of the source?
    pub anchor: RegionAnchor,

    pub out_label: String,
}

impl Step for Region {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> anyhow::Result<()> {
        self.resolved_source = Some(ResolvedSourceNoAll::parse(&self.source, input_def)?);
        Ok(())
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}
