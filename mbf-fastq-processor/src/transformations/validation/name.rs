#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::config::deser::single_u8_from_string;
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateName {
    #[serde(default, deserialize_with = "single_u8_from_string")]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl Step for ValidateName {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if input_def.segment_count() <= 1 {
            bail!(
                "ValidateName requires at least two input segments (e.g., read1 and read2) to compare read names. Found only {} segment(s).",
                input_def.segment_count()
            );
        }
        Ok(())
    }

    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!(
            "ValidateName should have been expanded into SpotCheckReadPairing before execution"
        );
    }
}
