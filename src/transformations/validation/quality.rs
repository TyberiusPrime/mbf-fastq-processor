#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::{Step, apply_in_place_wrapped_plus_all};
use crate::config::{PhredEncoding, SegmentIndexOrAll, SegmentOrAll};
use crate::transformations::prelude::*;
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ValidateQuality {
    pub encoding: PhredEncoding,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
}

impl Step for ValidateQuality {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        let (lower, upper) = self.encoding.limits();
        apply_in_place_wrapped_plus_all(
            self.segment_index.unwrap(),
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
            &mut block,
        );
        match res {
            Ok(()) => Ok((block, true)),
            Err(e) => Err(e),
        }
    }
}
