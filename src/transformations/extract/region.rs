#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{Demultiplexed, config::Segment};
use anyhow::Result;
use serde_valid::Validate;

use super::super::{RegionDefinition, Step, Transformation};

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub start: usize,
    #[serde(alias = "length")]
    pub len: usize,
    #[serde(alias = "segment")]
    pub source: Segment,
    pub label: String,
}

impl Step for Region {
    // a white lie. It's ExtractRegions that sets this tag.
    // But validation happens before the expansion of Transformations
    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        let mut regions = vec![RegionDefinition {
            source: self.source.clone(),
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
