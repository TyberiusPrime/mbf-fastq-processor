#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{Step, apply_in_place_wrapped_plus_all};
use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll, deser::bstring_from_string},
    demultiplex::Demultiplexed,
};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateSeq {
    #[serde(deserialize_with = "bstring_from_string")]
    pub allowed: BString,

    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for ValidateSeq {
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
        apply_in_place_wrapped_plus_all(
            self.segment_index.as_ref().unwrap(),
            |read| {
                assert!(
                    !read.seq().iter().any(|x| !self.allowed.contains(x)),
                    "Invalid base found in sequence: {:?} {:?}",
                    std::str::from_utf8(read.name()),
                    std::str::from_utf8(read.seq())
                );
            },
            &mut block,
        );

        (block, true)
    }
}
