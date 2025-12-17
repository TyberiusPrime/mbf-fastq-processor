#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{
    dna::{HitRegion, TagValue},
    transformations::{NewLocation, filter_tag_locations, filter_tag_locations_beyond_read_length},
};

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, Copy, JsonSchema)]
pub enum Direction {
    #[serde(alias = "start")]
    Start,
    #[serde(alias = "end")]
    End,
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimAtTag {
    in_label: String,
    direction: Direction,
    keep_tag: bool,
}

impl Step for TrimAtTag {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        for transformation in all_transforms {
            if let Transformation::ExtractRegions(extract_region_config) = transformation
                && extract_region_config.out_label == self.in_label
                && extract_region_config.regions.len() != 1
            {
                bail!(
                    "ExtractRegions and TrimAtTag only work together on single-entry regions. Label involved: {}",
                    self.in_label
                );
            }
        }
        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    #[allow(clippy::too_many_lines)]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);
        block.apply_mut_with_tag(
            self.in_label.as_str(),
            |reads, tag_hit| {
                if let Some(hit) = tag_hit.as_sequence() {
                    if hit.0.len() > 1 {
                                *error_encountered.borrow_mut() = Some(
                                "TrimAtTag only supports Tags that cover one single region. Could be extended to multiple hits within one target, but not to multiple hits in multiple targets.".to_string());
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

        let mut cut_locations: Vec<TagValue> = {
            block
                .tags
                .extract_if(|k, _v| k == &self.in_label)
                .next()
                .map(|(_k, v)| v)
                .expect("in_label tag must exist in block")
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
                            if let Some(hits) = cls.as_sequence()
                                && !hits.0.is_empty()
                                && let Some(trim_location) = &hits.0[0].location
                            {
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

                            NewLocation::Keep
                        },
                        None,
                    );
                }
            }
        }
        //now remove all locations from cut_locations
        if self.direction == Direction::Start {
            if self.keep_tag {
                //guess they're 0..len now.
                for cls in &mut cut_locations {
                    if let Some(hits) = cls.as_sequence_mut() {
                        for hit in &mut hits.0 {
                            if let Some(location) = &mut hit.location {
                                location.start = 0;
                            }
                        }
                    }
                }
            } else {
                for cls in &mut cut_locations {
                    if let Some(hits) = cls.as_sequence_mut() {
                        for hit in &mut hits.0 {
                            hit.location = None;
                        }
                    }
                }
            }
        } else if self.keep_tag {
            //do nothing, they're still good
        } else {
            for cls in &mut cut_locations {
                if let Some(hits) = cls.as_sequence_mut() {
                    for hit in &mut hits.0 {
                        hit.location = None;
                    }
                }
            }
        }

        block.tags.insert(self.in_label.clone(), cut_locations);

        Ok((block, true))
    }
}
