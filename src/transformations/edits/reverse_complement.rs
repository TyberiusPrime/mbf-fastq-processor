#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{NewLocation, Step, apply_in_place_wrapped, filter_tag_locations};
use crate::{
    config::{Segment, SegmentIndex},
    demultiplex::Demultiplexed,
    dna::HitRegion,
};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReverseComplement {
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,
}

impl Step for ReverseComplement {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(
            self.segment_index.as_ref().unwrap(),
            |read| read.reverse_complement(),
            &mut block,
        );

        filter_tag_locations(
            &mut block,
            self.segment_index.as_ref().unwrap(),
            |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            segment_index: location.segment_index.clone(),
                        },
                        new_seq.into(),
                    )
                }
            },
        );

        (block, true)
    }
}
