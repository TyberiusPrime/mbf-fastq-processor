#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::config::deser::single_u8_from_string;
use crate::transformations::prelude::*;

/// Validate that read names between segments match
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidateName {
    #[serde(default, deserialize_with = "single_u8_from_string")]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl FromTomlTableNested for ValidateName {
    fn from_toml_table(table: &toml_edit::Table, mut helper: TableErrorHelper) -> TomlResult<Self> {
        let readname_end_char: TomlResult<Option<(&str, u8)>> =
            helper.get_opt_u8_from_char_or_number(&["read_name_end_char", "readname_end_char"][..],
            Some(33), //ascii
            Some(126),

        );
        helper.deny_unknown()?;

        if helper.errors.borrow().get_segment_order().len() <= 1 {
            return helper.add_table::<Self>(table,
                "Too few segments available",
                "ValidateName requires at least two input segments (e.g., read1 and read2) to compare read names. Add segments or remove validation step." 
            );
        }
        Ok(ValidateName {
            readname_end_char: readname_end_char?.map(|x| x.1),
        })
    }
}

impl Step for ValidateName {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if input_def.segment_count() <= 1 {}
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
