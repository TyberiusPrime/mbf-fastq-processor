#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use serde_valid::Validate;

/// Define a region by coordinates
#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub start: isize,
    #[serde(alias = "length")]
    pub len: usize,

    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[serde(alias = "segment")]
    pub source: String,

    #[serde(default)]
    #[serde(skip)]
    pub resolved_source: Option<ResolvedSourceNoAll>,

    /// Is the region from the `Start` or the `End` of the source?
    pub anchor: super::super::RegionAnchor,

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
