#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{apply_in_place_wrapped_plus_all, Step, Transformation};
use crate::{
    config::{deser::bstring_from_string, SegmentOrAll},
    demultiplex::Demultiplexed,
};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateSeq {
    #[serde(deserialize_with = "bstring_from_string")]
    pub allowed: BString,
    #[eserde(compat)]
    pub segment: SegmentOrAll,
}

impl Step for ValidateSeq {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
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
