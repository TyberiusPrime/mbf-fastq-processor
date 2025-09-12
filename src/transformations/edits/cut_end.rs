#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{Step, apply_in_place, filter_tag_locations_beyond_read_length};
use crate::{
    config::{Segment, SegmentIndex},
    demultiplex::Demultiplexed,
};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutEnd {
    n: usize,
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
}

impl Step for CutEnd {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(
            self.segment_index.as_ref().unwrap(),
            |read| read.cut_end(self.n),
            &mut block,
        );
        filter_tag_locations_beyond_read_length(&mut block, self.segment_index.as_ref().unwrap());

        (block, true)
    }
}
