#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::config::deser::{single_u8_from_string, tpd_extract_u8_from_byte_or_char};
use crate::transformations::prelude::*;

/// Validate that read names between segments match
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateName {
    #[tpd_adapt_in_verify]
    #[tpd_alias("read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl VerifyFromToml for PartialValidateName {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.readname_end_char = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_readname_end_char(helper, false, false),
            self.tpd_get_readname_end_char(helper, false, false),
            false,
            helper,
        )
        .into_optional();
        self
    }
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
