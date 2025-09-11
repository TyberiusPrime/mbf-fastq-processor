#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{filter_tag_locations_all_targets, NewLocation, Segment, Step, Transformation};
use crate::{demultiplex::Demultiplexed, dna::HitRegion};
use anyhow::{bail, Result};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SwapR1AndR2 {
    #[eserde(compat)]
    segment_a: Segment,
    #[eserde(compat)]
    segment_b: Segment,
}

impl Step for SwapR1AndR2 {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_a.validate(input_def)?;
        self.segment_b.validate(input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block
            .segments
            .swap(self.segment_a.get_index(), self.segment_b.get_index());
        let index_a = self.segment_a.get_index();
        let index_b = self.segment_b.get_index();
        let name_a = self.segment_a.get_name().to_string();
        let name_b = self.segment_b.get_name().to_string();

        filter_tag_locations_all_targets(
            &mut block,
            |location: &HitRegion, _pos: usize| -> NewLocation {
                NewLocation::New(HitRegion {
                    start: location.start,
                    len: location.len,
                    segment: match location.segment {
                        Segment::Indexed(index, _) if index == index_a => {
                            Segment::Indexed(index_b, name_b.clone())
                        }
                        Segment::Indexed(index, _) if index == index_b => {
                            Segment::Indexed(index_a, name_a.clone())
                        }
                        _ => location.segment.clone(), // others unchanged
                    },
                })
            },
        );

        (block, true)
    }
}
