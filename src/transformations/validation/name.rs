#![allow(clippy::unnecessary_wraps)] // eserde false positives
use super::Step;
use crate::config::deser::single_u8_from_string;
use anyhow::{Result, bail};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateName {
    #[serde(default, deserialize_with = "single_u8_from_string")]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl Step for ValidateName {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if input_def.segment_count() <= 1 {
            bail!("ValidateName requires at least two input segments");
        }
        Ok(())
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &crate::demultiplex::Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        unreachable!(
            "ValidateName should have been expanded into SpotCheckReadPairing before execution"
        );
    }
}
