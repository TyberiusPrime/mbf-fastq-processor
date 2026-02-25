#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::transformations::prelude::*;

/// Validate that read names between segments match
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateName {
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub readname_end_char: Option<u8>,
}

impl VerifyIn<PartialConfig> for PartialValidateName {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(input_config) = parent.input.as_ref() {
            if input_config.get_segment_order().len() < 2 {
                return Err(ValidationFailure::new(
                    "ValidateName requires at least two input segments",
                    Some("Check your [input] section or remove the step"),
                ));
            }
        }
        Ok(())
    }
}

impl Step for ValidateName {
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
