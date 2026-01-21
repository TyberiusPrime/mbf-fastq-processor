#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Lowercase {
    #[serde(alias = "segment")]
    #[serde(alias = "source")]
    pub target: String,

    #[serde(default)]
    pub if_tag: Option<String>,
}

impl Step for Lowercase {
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
