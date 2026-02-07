#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::config::deser::{bstring_from_string, tpd_adapt_bstring};

/// Validate that the sequence is only consisting of the specified bases
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateSeq {
    #[tpd_with(tpd_adapt_bstring)]
    #[schemars(with = "String")]
    pub allowed: BString,

    #[tpd_default]
    segment: SegmentOrAll,
    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for ValidateSeq {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut res = Ok(());
        block.apply_in_place_wrapped_plus_all(
            self.segment_index
                .expect("segment_index must be set during initialization"),
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
