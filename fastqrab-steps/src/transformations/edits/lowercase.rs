use crate::transformations::prelude::*;

/// Convert a read, name, tag into lower case
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Lowercase {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "source")]
    #[tpd(alias = "segment")]
    pub target: ResolvedSourceAll,

    #[tpd(alias = "if_label")]
    pub if_tag: Option<ConditionalTagLabel>,
}

impl VerifyIn<PartialConfig> for PartialLowercase {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialLowercase> {}

impl Step for Lowercase {
    // cov:excl-start
    fn apply(
        &self,
        _block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        unreachable!();
    }
    // cov:excl-stop
}
