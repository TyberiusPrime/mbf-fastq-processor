#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Lowercase {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias="source")]
    #[tpd(alias="segment")]
    pub target: ResolvedSourceAll,

    #[serde(default)]
    pub if_tag: Option<String>,
}

impl VerifyIn<PartialConfig> for PartialLowercase {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        Ok(())
    }
}

impl Step for Lowercase {

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
