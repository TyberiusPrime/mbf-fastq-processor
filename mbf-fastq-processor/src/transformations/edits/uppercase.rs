#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

#[derive( Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Uppercase {
    #[tpd_alias("segment")]
    #[tpd_alias("source")]
    pub target: String,

    pub if_tag: Option<String>,
}

impl Step for Uppercase {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        ResolvedSourceAll::parse(&self.target, input_def)?;
        Ok(())
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        Ok((block, true))
    }
}
