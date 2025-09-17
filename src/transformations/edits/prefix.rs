#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{
    NewLocation, Step, Transformation, apply_in_place_wrapped, filter_tag_locations,
};
use crate::{
    config::{
        Segment, SegmentIndex,
        deser::{bstring_from_string, dna_from_string},
    },
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::{Result, bail};
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    #[serde(deserialize_with = "dna_from_string")]
    pub seq: BString,
    #[serde(deserialize_with = "bstring_from_string")]
    //we don't check the quality. It's on you if you
    //write non phred values in there
    pub qual: BString,
}

impl Step for Prefix {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
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
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        apply_in_place_wrapped(
            self.segment_index.unwrap(),
            |read| read.prefix(&self.seq, &self.qual),
            &mut block,
        );
        let prefix_len = self.seq.len();

        filter_tag_locations(
            &mut block,
            self.segment_index.unwrap(),
            |location: &HitRegion, _pos, _seq, _read_len: usize| -> NewLocation {
                {
                    NewLocation::New(HitRegion {
                        start: location.start + prefix_len,
                        len: location.len,
                        segment_index: location.segment_index,
                    })
                }
            },
        );

        Ok((block, true))
    }
}
