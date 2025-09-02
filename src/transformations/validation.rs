use super::{apply_in_place_wrapped_plus_all, validate_target_plus_all, Step, Transformation};
use crate::{
    config::{deser::u8_from_string, TargetPlusAll},
    demultiplex::Demultiplexed,
};
use anyhow::Result;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateSeq {
    #[serde(deserialize_with = "u8_from_string")]
    pub allowed: Vec<u8>,
    pub target: TargetPlusAll,
}

impl Step for ValidateSeq {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
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

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidatePhred {
    pub target: TargetPlusAll,
}

impl Step for ValidatePhred {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
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
