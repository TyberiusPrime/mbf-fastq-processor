#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::{bstring_from_string, tpd_adapt_bstring};

/// Validate that the sequence is only consisting of the specified bases
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateSeq {
    #[tpd(with="tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub allowed: BString,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialValidateSeq {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl Step for ValidateSeq {

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        block.apply_in_place_wrapped_plus_all(
            self.segment,
            |read| {
                if res.is_ok() && read.seq().iter().any(|x| !self.allowed.contains(x)) {
                    res = Err(anyhow::anyhow!(
                        "Invalid base found in read named '{}', sequence: '{}' Bytes: {:?}",
                        BString::from(read.name()),
                        BString::from(read.seq()),
                        read.seq()
                    ));
                }
            },
            None,
        );
        match res {
            Ok(()) => Ok((block, true)),
            Err(e) => Err(e),
        }
    }
}
