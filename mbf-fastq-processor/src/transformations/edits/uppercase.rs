#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

#[derive( Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Uppercase {
    #[tpd(alias="segment")]
    #[tpd(alias="source")]
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub target: ResolvedSourceAll,

    pub if_tag: Option<String>,
}

impl VerifyIn<PartialConfig> for PartialUppercase {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        Ok(())
    }
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
