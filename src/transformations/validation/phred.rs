#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{apply_in_place_wrapped_plus_all, Step, Transformation};
use crate::{config::{SegmentIndexOrAll, SegmentOrAll}, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidatePhred {

    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for ValidatePhred {
    
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
                assert!(
                    !read.qual().iter().any(|x| *x < 33 || *x > 74),
                    "Invalid phred quality found. Expected 33..=74 (!..J) : {:?} {:?}",
                    std::str::from_utf8(read.name()),
                    std::str::from_utf8(read.qual())
                );
            },
            &mut block,
        );

        (block, true)
    }
}
