#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{apply_in_place_wrapped_plus_all, Step};
use crate::{
    config::{ SegmentIndexOrAll, SegmentOrAll},
    demultiplex::Demultiplexed,
};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowercaseSequence {
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for LowercaseSequence {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_plus_all(
            self.segment_index.as_ref().unwrap(),
            |read| {
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_lowercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
        );

        (block, true)
    }
}
