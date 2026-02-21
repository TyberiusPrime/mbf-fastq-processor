#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::config::PhredEncoding;
use crate::transformations::prelude::*;

/// Validate that quality scores are within Sanger (PHRED 33) range.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateQuality {
    pub encoding: PhredEncoding,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialValidateQuality {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl Step for ValidateQuality {

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        let (lower, upper) = self.encoding.limits();
        block.apply_in_place_wrapped_plus_all(
            self.segment,
            |read| {
                if res.is_ok() && read.qual().iter().any(|x| *x < lower || *x > upper) {
                    res = Err(anyhow::anyhow!(
                        "Invalid phred quality found. Expected {lower}..={upper} ({}..={}) : Error in read named '{}', Quality: '{}' Bytes: {:?}",
                        lower as char,
                        upper as char,
                        BString::from(read.name()),
                        BString::from(read.qual()),
                        read.qual(),
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
