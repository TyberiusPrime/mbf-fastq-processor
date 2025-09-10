#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    apply_in_place_wrapped_plus_all,  Step, Transformation,
};
use crate::{config::SegmentOrAll, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct UppercaseSequence {
    segment: SegmentOrAll,
}

impl Step for UppercaseSequence {
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
        apply_in_place_wrapped_plus_all(
            &self.segment,
            |read| {
                let seq = read.seq().to_vec();
                let new_seq: Vec<u8> = seq.iter().map(|&b| b.to_ascii_uppercase()).collect();
                read.replace_seq(new_seq, read.qual().to_vec());
            },
            &mut block,
        );

        (block, true)
    }
}
