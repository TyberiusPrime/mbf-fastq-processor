use bstr::ByteSlice;

use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_bstring;

/// Validate that the sequence is only consisting of the specified bases
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateSeq {
    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub allowed: BString,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialValidateSeq {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialValidateSeq> {
    //default is ok, no tags
}

impl Step for ValidateSeq {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        for segment in block.iter_matching_segments(self.segment) {
            for read in segment.iter() {
                if read.seq.iter().any(|x| !self.allowed.contains(x)) {
                    bail!(
                        "Invalid base found in read named '{}'\n\
                        Accepted: any of '{}'.\n\
                        Read sequence : '{}' Bytes: {:?}",
                        read.name,
                        self.allowed,
                        read.seq,
                        read.seq.as_bytes()
                    );
                }
            }
        }
        Ok((block, true))
    }
}
