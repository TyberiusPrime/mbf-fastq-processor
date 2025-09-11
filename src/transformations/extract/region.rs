#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    Demultiplexed,
    config::{Segment, SegmentIndex},
};
use anyhow::Result;
use serde_valid::Validate;

use super::super::{RegionDefinition, Step};

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub start: usize,
    #[serde(alias = "length")]
    pub len: usize,
    #[serde(alias = "segment")]
    pub segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndex>,

    pub label: String,
}

impl Step for Region {
    // a white lie. It's ExtractRegions that sets this tag.
    // But validation happens before the expansion of Transformations

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        let mut regions = vec![RegionDefinition {
            segment: self.segment.clone(),
            segment_index: self.segment_index.clone(),
            start: self.start,
            length: self.len,
        }];
        super::super::validate_regions(&mut regions, input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        panic!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}
