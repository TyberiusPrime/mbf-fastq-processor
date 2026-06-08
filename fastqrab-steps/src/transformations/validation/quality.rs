use crate::transformations::prelude::*;
use fastqrab_config::fileformats::PhredEncoding;

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

impl TagUser for PartialTaggedVariant<PartialValidateQuality> {
    //default is ok, no tags
}

impl Step for ValidateQuality {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let (lower, upper) = self.encoding.limits();
        for segment in block.iter_matching_segments(self.segment) {
            for read in segment.iter() {
                if read.qual.iter().any(|x| *x < lower || *x > upper) {
                    bail!(
                        "Invalid phred quality found. Expected {lower}..={upper} ({}..={}) : Error in read named '{}', Quality: '{}' Bytes: {:?}",
                        lower as char,
                        upper as char,
                        read.name,
                        read.qual,
                        read.qual,
                    );
                }
            }
        }
        Ok((block, true))
    }
}
