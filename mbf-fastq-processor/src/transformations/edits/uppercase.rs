#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Uppercase {
    #[serde(alias = "segment")]
    #[serde(alias = "source")]
    pub target: String,

    #[serde(default)]
    pub if_tag: Option<String>,
}

impl Step for Uppercase {
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
