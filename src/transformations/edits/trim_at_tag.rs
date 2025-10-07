#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    dna::{HitRegion, TagValue},
    transformations::{filter_tag_locations, filter_tag_locations_beyond_read_length, NewLocation},
    Demultiplexed,
};
use anyhow::{bail, Result};

use super::super::{Step, Transformation};

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, Copy)]
pub enum Direction {
    #[serde(alias = "start")]
    Start,
    #[serde(alias = "end")]
    End,
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TrimAtTag {
    label: String,
    direction: Direction,
    keep_tag: bool,
}

impl Step for TrimAtTag {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        for transformation in all_transforms {
            if let Transformation::ExtractRegions(extract_region_config) = transformation {
                if extract_region_config.label == self.label
                    && extract_region_config.regions.len() != 1
                {
                    bail!(
                        "ExtractRegions and TrimAtTag only work together on single-entry regions. Label involved: {}",
                        self.label
                    );
                }
            }
        }
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);
        block.apply_mut_with_tag(
            self.label.as_str(),
            |reads, tag_hit| {
                if let Some(hit) = tag_hit.as_sequence() {
                    if hit.0.len() > 1 {
                                *error_encountered.borrow_mut() = Some(format!(
                                "TrimAtTag only supports Tags that cover one single region. Could be extended to multiple hits within one target, but not to multiple hits in multiple targets."));
                        return;
                    }
                    let region = &hit.0[0];
                    let location = region.location.as_ref().expect("TrimTag only works on regions with location data. Might have been lost by subsequent transformations?");
                    let read = &mut reads[location.segment_index.get_index()];
                    match (self.direction, self.keep_tag) {
                        (Direction::Start, true) => read.cut_start(location.start),
                        (Direction::Start, false) => read.cut_start(location.start + location.len),
                        (Direction::End, true) => read.max_len(location.start + location.len),
                        (Direction::End, false) => read.max_len(location.start),
                    }
                }
            },
        );
        if let Some(error_msg) = error_encountered.borrow().as_ref() {
            return Err(anyhow::anyhow!("{error_msg}"));
        }


        let cut_locations: Vec<TagValue> = {
            let tags = block.tags.as_ref().unwrap();
            tags.get(&self.label).unwrap().clone()
        };
        if let Some(target) = cut_locations
            .iter()
            //first not none
            .filter_map(|tag_val| tag_val.as_sequence())
            // that has locations
            .filter_map(|hit| hit.0.first())
            //and the target from that
            .filter_map(|hit| hit.location.as_ref())
            .map(|location| &location.segment_index)
            .next()
        //otherwise, we didn't have a single hit, no need to filter anything...
        {
            match (self.direction, self.keep_tag) {
                (Direction::End, _) => {
                    filter_tag_locations_beyond_read_length(&mut block, *target);
                }
                (Direction::Start, keep_tag) => {
                    filter_tag_locations(
                        &mut block,
                        *target,
                        |location: &HitRegion, pos: usize, _seq, _read_len: usize| -> NewLocation {
                            let cls = &cut_locations[pos];
                            if let Some(hits) = cls.as_sequence() {
                                if !hits.0.is_empty() {
                                    if let Some(trim_location) = &hits.0[0].location {
                                        let cut_point = if keep_tag {
                                            trim_location.start
                                        } else {
                                            trim_location.start + trim_location.len
                                        };
                                        //todo: this could use some more test cases
                                        if location.start < cut_point {
                                            return NewLocation::Remove;
                                        } else {
                                            return NewLocation::New(HitRegion {
                                                start: location.start - cut_point,
                                                len: location.len,
                                                segment_index: location.segment_index,
                                            });
                                        }
                                    }
                                }
                            }
                            NewLocation::Keep
                        },
                    );
                }
            }
        }

        Ok((block, true))
    }
}
