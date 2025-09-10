#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{apply_in_place_wrapped, Segment, Step, Transformation};
use crate::{
    config::deser::{bstring_from_string, dna_from_string},
    demultiplex::Demultiplexed,
};
use anyhow::{bail, Result};
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Postfix {
    #[eserde(compat)]
    pub segment: Segment,
    #[serde(deserialize_with = "dna_from_string")]
    pub seq: BString,
    #[serde(deserialize_with = "bstring_from_string")]
    //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: BString,
}

impl Step for Postfix {
    fn validate_others(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.seq.len() != self.qual.len() {
            bail!("Seq and qual must be the same length");
        }
        Ok(())
    }
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment.validate(input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            &self.segment,
            |read| read.postfix(&self.seq, &self.qual),
            &mut block,
        );
        // postfix doesn't change tags.
        (block, true)
    }
}
