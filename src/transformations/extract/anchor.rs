#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::{cell::Cell, path::Path};

use crate::transformations::TagValueType;
use crate::{config::deser::bstring_from_string, dna::Hits, Demultiplexed};
use anyhow::{bail, Result};

use super::super::{tag::default_region_separator, Step};
use super::extract_tags;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Anchor {
    input_label: String,
    #[eserde(compat)]
    pub regions: Vec<(isize, usize)>,

    #[serde(deserialize_with = "bstring_from_string")]
    #[serde(default = "default_region_separator")]
    pub region_separator: BString,

    label: String,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    left_most: isize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    right_most: isize,
}

impl Step for Anchor {
    fn init(
        &mut self,
        _input_info: &super::super::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        self.left_most = self
            .regions
            .iter()
            .map(|(region_start, _region_len)| *region_start)
            .min()
            .unwrap(); // we have at least one region
        self.right_most = self
            .regions
            .iter()
            .map(|(region_start, region_len)| {
                let region_len: isize = (*region_len)
                    .try_into()
                    .expect("region length > isize limit");
                *region_start + region_len
            }) // we validate below
            .max()
            .unwrap();
        Ok(None)
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        if self.regions.is_empty() {
            bail!("ExtractAnchor requires at least one region to extract.");
        }
        for (_start, len) in &self.regions {
            if *len == 0 {
                bail!(
                    "ExtractAnchor requires regions with non-zero length. Found a region with length 0."
                );
            }
        }
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn uses_tags(&self) -> Option<Vec<(String, TagValueType)>> {
        vec![(self.input_label.clone(), TagValueType::Location)].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        // Get the input tag data
        let input_tag_data = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.input_label))
            .expect("Tag missing. Should have been caught earlier.");

        // Determine the target from the first available tag with location
        let segment = input_tag_data
            .iter()
            .filter_map(|tag_val| tag_val.as_sequence())
            .filter_map(|hits| hits.0.first())
            .filter_map(|hit| hit.location.as_ref())
            .map(|location| location.segment_index)
            .next();

        if let Some(segment) = segment {
            // Clone the input tag data so we can access it by index
            let input_tag_data_vec: Vec<_> = input_tag_data.clone();

            // Create an index counter to track which read we're processing
            let read_index = Cell::new(0);

            extract_tags(&mut block, segment, &self.label, |read| {
                let seq = read.seq();
                let current_index = read_index.get();
                read_index.set(current_index + 1);

                // Find the corresponding tag entry for this read
                if let Some(tag_val) = input_tag_data_vec.get(current_index) {
                    if let Some(hits) = tag_val.as_sequence() {
                        // Get the leftmost position from the tag
                        let leftmost_pos = hits
                            .0
                            .iter()
                            .filter_map(|hit| hit.location.as_ref())
                            .map(|location| location.start)
                            .min();

                        if let Some(anchor_pos) = leftmost_pos {
                            let anchor_pos: isize = anchor_pos
                                .try_into()
                                .expect("anchor pos beyond isize limit");
                            let start = anchor_pos + self.left_most;
                            if start < 0 {
                                return None;
                            }
                            let stop = anchor_pos + self.right_most;
                            if stop
                                > seq
                                    .len()
                                    .try_into()
                                    .expect("read length beyond isize limit")
                            {
                                return None;
                            }
                            assert!(stop > start);
                            let len = stop - start;

                            let mut replacement: BString = BString::default();
                            let mut first = true;
                            for (region_start, region_len) in &self.regions {
                                if !first {
                                    replacement.extend(self.region_separator.iter());
                                }
                                first = false;
                                let absolute_region_start: usize = (anchor_pos + region_start)
                                    .try_into()
                                    .expect("region start beyond usize limit");
                                let absolute_region_end = absolute_region_start + region_len;
                                //will be within read.seq() due to the left_most, right_most checks above.
                                replacement
                                    .extend(&seq[absolute_region_start..absolute_region_end]);
                            }
                            return Some(Hits::new(
                                start.try_into().expect("usize limit"),
                                len.try_into().expect("usize limit"),
                                segment,
                                replacement,
                            ));
                        }
                    }
                }
                None
            });
        }

        Ok((block, true))
    }
}
