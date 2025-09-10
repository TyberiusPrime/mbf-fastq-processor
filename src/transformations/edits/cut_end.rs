#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    apply_in_place, filter_tag_locations_beyond_read_length, Segment, Step, Transformation,
};
use crate::demultiplex::Demultiplexed;
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CutEnd {
    n: usize,
    segment: Segment,
}

impl Step for CutEnd {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        self.segment.validate(input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place(&self.segment, |read| read.cut_end(self.n), &mut block);
        filter_tag_locations_beyond_read_length(&mut block, &self.segment);

        (block, true)
    }
}
