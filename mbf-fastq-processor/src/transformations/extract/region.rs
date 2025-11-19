#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::super::RegionDefinition;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Region {
    /// 0-based start position in the sequence
    pub start: usize,
    /// Length of the region to extract
    #[serde(alias = "length")]
    pub len: usize,
    /// Any of your input segments (default: read1)
    #[serde(alias = "segment")]
    #[serde(default)]
    pub segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndex>,

    /// Label to store the extracted tag under
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

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        let mut regions = vec![RegionDefinition {
            segment: self.segment.clone(),
            segment_index: self.segment_index,
            start: self.start,
            length: self.len,
        }];
        super::super::validate_regions(&mut regions, input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
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
