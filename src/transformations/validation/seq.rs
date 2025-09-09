#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{Step, Transformation, apply_in_place_wrapped_plus_all, validate_target_plus_all};
use crate::{
    config::{TargetPlusAll, deser::bstring_from_string},
    demultiplex::Demultiplexed,
};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateSeq {
    #[serde(deserialize_with = "bstring_from_string")]
    pub allowed: BString,
    pub target: TargetPlusAll,
}

impl Step for ValidateSeq {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target_plus_all(self.target, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_plus_all(
            self.target,
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
