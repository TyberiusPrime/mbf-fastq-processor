#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{apply_in_place_wrapped, filter_tag_locations, NewLocation, Segment, Step};
use crate::{demultiplex::Demultiplexed, dna::HitRegion};
use anyhow::Result;
use bstr::BString;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReverseComplement {
    #[eserde(compat)]
    pub segment: Segment,
}

impl Step for ReverseComplement {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment.validate(input_def)
    }

    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
                                                         // enough
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped(&self.segment, |read| read.reverse_complement(), &mut block);

        filter_tag_locations(
            &mut block,
            &self.segment,
            |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                {
                    let new_start = read_len - (location.start + location.len);
                    let new_seq = crate::dna::reverse_complement_iupac(seq);
                    NewLocation::NewWithSeq(
                        HitRegion {
                            start: new_start,
                            len: location.len,
                            segment: location.segment.clone(),
                        },
                        new_seq.into(),
                    )
                }
            },
        );

        (block, true)
    }
}
