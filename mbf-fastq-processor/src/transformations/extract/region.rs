#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use serde_valid::Validate;

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
    pub resolved_source: Option<ResolvedSource>,

    /// Is the region from the `Start` or the `End` of the source?
    pub anchor: super::super::RegionAnchor,

    pub out_label: String,
}

impl Step for Region {
    // a white lie. It's ExtractRegions that sets this tag.
    // But validation happens before the expansion of Transformations

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> anyhow::Result<()> {
        self.resolved_source = Some(ResolvedSource::parse(&self.source, input_def)?);
        Ok(())
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        panic!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}
