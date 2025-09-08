#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use ex::Wrapper;
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::{
    Demultiplexed,
    config::{
        Target, TargetPlusAll,
        deser::{bstring_from_string, u8_from_char_or_number, u8_regex_from_string},
    },
    dna::{Anchor, Hit, HitRegion, Hits, TagValue},
    io,
    transformations::filter_tag_locations_all_targets,
};
use anyhow::{Result, bail};
use serde_valid::Validate;

use super::{
    FinalizeReportResult, NewLocation, RegionDefinition, Step, Transformation, extract_regions,
    filter_tag_locations, filter_tag_locations_beyond_read_length,
};

fn default_region_separator() -> BString {
    b"-".into()
}
fn default_target_read1() -> TargetPlusAll {
    TargetPlusAll::Read1
}

fn extract_tags(
    target: Target,
    label: &str,
    f: impl Fn(&mut io::WrappedFastQRead) -> Option<Hits>,
    block: &mut io::FastQBlocksCombined,
) {
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }
    let mut out = Vec::new();

    let f2 = |read: &mut io::WrappedFastQRead| {
        out.push(match f(read) {
            Some(hits) => TagValue::Sequence(hits),
            None => TagValue::Missing,
        });
    };

    match target {
        Target::Read1 => block.read1.apply(f2),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f2),
    };
    block.tags.as_mut().unwrap().insert(label.to_string(), out);
}

pub(crate) fn extract_numeric_tags<F>(
    target: Target,
    label: &str,
    extractor: F,
    block: &mut io::FastQBlocksCombined,
) where
    F: Fn(&io::WrappedFastQRead) -> f64,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    let mut values = Vec::new();
    let f = |read: &mut io::WrappedFastQRead| {
        values.push(TagValue::Numeric(extractor(read)));
    };

    match target {
        Target::Read1 => block.read1.apply(f),
        Target::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
        Target::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply(f),
    };

    block
        .tags
        .as_mut()
        .unwrap()
        .insert(label.to_string(), values);
}

pub(crate) fn extract_numeric_tags_plus_all<F>(
    target: TargetPlusAll,
    label: &str,
    extractor_single: F,
    extractor_all: impl Fn(
        &io::WrappedFastQRead,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
        Option<&io::WrappedFastQRead>,
    ) -> f64,
    block: &mut io::FastQBlocksCombined,
) where
    F: Fn(&io::WrappedFastQRead) -> f64,
{
    if block.tags.is_none() {
        block.tags = Some(HashMap::new());
    }

    if let Ok(target) = target.try_into() as Result<Target, _> {
        // Handle single target case
        extract_numeric_tags(target, label, extractor_single, block);
    } else {
        // Handle "All" target case
        let mut values = Vec::new();
        let mut block_iter = block.get_pseudo_iter();
        while let Some(molecule) = block_iter.pseudo_next() {
            let value = extractor_all(
                &molecule.read1,
                molecule.read2.as_ref(),
                molecule.index1.as_ref(),
                molecule.index2.as_ref(),
            );
            values.push(TagValue::Numeric(value));
        }
        block
            .tags
            .as_mut()
            .unwrap()
            .insert(label.to_string(), values);
    }
}

///Extract a IUPAC described sequence from the read. E.g. an adapter.
///Can be at the start (anchor = Left, the end (anchor = Right),
///or anywhere (anchor = Anywhere) within the read.
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractIUPAC {
    #[serde(deserialize_with = "crate::config::deser::iupac_from_string")]
    search: BString,
    pub target: Target,
    anchor: Anchor,
    label: String,
    #[serde(default)] // 0 is fine.
    max_mismatches: u8,
}

impl Step for ExtractIUPAC {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(
            self.target,
            &self.label,
            |read| read.find_iupac(&self.search, self.anchor, self.max_mismatches, self.target),
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegex {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    pub replacement: BString,
    label: String,
    pub target: Target,
}

impl Step for ExtractRegex {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target(self.target, input_def)?;
        // regex treats  $1_$2 as a group named '1_'
        // and just silently omits it.
        // Let's remove that foot gun. I'm pretty sure you can work around it if
        // you have a group named '1_'...
        let group_hunting_regexp = regex::bytes::Regex::new("[$]\\d+_").unwrap();
        if group_hunting_regexp.is_match(&self.replacement) {
            bail!(
                "Replacement string for ExtractRegex contains a group reference like  '$1_'. This is a footgun, as it would be interpreted as a group name, not the expected $1 followed by '_' . Please change the replacement string to use ${{1}}_."
            );
        }
        Ok(())
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(
            self.target,
            &self.label,
            |read| {
                let re_hit = self.search.captures(read.seq());
                if let Some(hit) = re_hit {
                    let mut replacement = Vec::new();
                    let g = hit.get(0).expect("Regex should always match");
                    hit.expand(&self.replacement, &mut replacement);
                    Some(Hits::new(
                        g.start(),
                        g.end() - g.start(),
                        self.target,
                        replacement.into(),
                    ))
                } else {
                    None
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractAnchor {
    input_label: String,
    #[eserde(compat)]
    pub regions: Vec<(isize, usize)>,

    #[serde(deserialize_with = "bstring_from_string")]
    #[serde(default = "default_region_separator")]
    pub region_separator: BString,

    label: String,
    #[serde(skip)]
    left_most: isize,
    #[serde(skip)]
    right_most: isize,
}

impl Step for ExtractAnchor {
    fn init(
        &mut self,
        _input_info: &super::InputInfo,
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

    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
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

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.input_label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        // Get the input tag data
        let input_tag_data = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.input_label))
            .expect("Tag missing. Should have been caught earlier.");

        // Determine the target from the first available tag with location
        let target = input_tag_data
            .iter()
            .filter_map(|tag_val| tag_val.as_sequence())
            .filter_map(|hits| hits.0.first())
            .filter_map(|hit| hit.location.as_ref())
            .map(|location| location.target)
            .next();

        if let Some(target) = target {
            // Clone the input tag data so we can access it by index
            let input_tag_data_vec: Vec<_> = input_tag_data.clone();

            // Create an index counter to track which read we're processing
            let read_index = Cell::new(0);

            extract_tags(
                target,
                &self.label,
                |read| {
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
                                    target,
                                    replacement,
                                ));
                            }
                        }
                    }
                    None
                },
                &mut block,
            );
        }

        (block, true)
    }
}

fn apply_in_place_wrapped_with_tag(
    target: TargetPlusAll,
    label: &str,
    block: &mut io::FastQBlocksCombined,
    f: impl Fn(&mut io::WrappedFastQReadMut, &TagValue),
) {
    match target {
        TargetPlusAll::Read1 => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f);
        }

        TargetPlusAll::Read2 => block
            .read2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index1 => block
            .index1
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::Index2 => block
            .index2
            .as_mut()
            .expect("Input def and transformation def mismatch")
            .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, f),
        TargetPlusAll::All => {
            block
                .read1
                .apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            if let Some(read2) = &mut block.read2 {
                read2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index1) = &mut block.index1 {
                index1.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
            if let Some(index2) = &mut block.index2 {
                index2.apply_mut_with_tag(block.tags.as_ref().unwrap(), label, &f);
            }
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FilterByTag {
    label: String,
    keep_or_remove: super::KeepOrRemove,
}

impl Step for FilterByTag {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let mut keep: Vec<bool> = block
            .tags
            .as_ref()
            .and_then(|tags| tags.get(&self.label))
            .expect("Tag not set? Should have been caught earlier in validation.")
            .iter()
            .map(|tag_val| !tag_val.is_missing())
            .collect();
        if self.keep_or_remove == super::KeepOrRemove::Remove {
            keep.iter_mut().for_each(|x| *x = !*x);
        }
        super::apply_bool_filter(&mut block, &keep);

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Eq, PartialEq, Copy)]
enum Direction {
    Start,
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

    fn validate(
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
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.apply_mut_with_tag(
            self.label.as_str(),
            |read1, read2, index1, index2, tag_hit| {
                if let Some(hit) = tag_hit.as_sequence() {
                    assert_eq!(hit.0.len(), 1, "TrimAtTag only supports Tags that cover one single region. Could be extended to multiple tags within one target, but not to multiple hits in multiple targets.");
                    let region = &hit.0[0];
                    let location = region.location.as_ref().expect("TrimTag only works on regions with location data. Might have been lost by subsequent transformations?");
                    let read = match location.target {
                        Target::Read1 => read1,
                        Target::Read2 => read2
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                        Target::Index1 => index1
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                        Target::Index2 => index2
                            .as_mut()
                            .expect("Input def and transformation def mismatch"),
                    };
                    match (self.direction, self.keep_tag) {
                        (Direction::Start, true) => read.cut_start(location.start),
                        (Direction::Start, false) => read.cut_start(location.start + location.len),
                        (Direction::End, true) => read.max_len(location.start + location.len),
                        (Direction::End, false) => read.max_len(location.start),
                    }
                }
            },
        );

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
            .map(|location| location.target)
            .next()
        //otherwise, we didn't have a single hit, no need to filter anything...
        {
            match (self.direction, self.keep_tag) {
                (Direction::End, _) => {
                    filter_tag_locations_beyond_read_length(&mut block, target);
                }
                (Direction::Start, keep_tag) => {
                    filter_tag_locations(
                        &mut block,
                        target,
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
                                        if location.start + location.len <= cut_point {
                                            return NewLocation::Remove;
                                        } else {
                                            return NewLocation::New(HitRegion {
                                                start: location.start - cut_point,
                                                len: location.len,
                                                target: location.target,
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

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegion {
    pub start: usize,
    #[serde(alias = "length")]
    pub len: usize,
    #[serde(alias = "target")]
    pub source: Target,
    pub label: String,
}

impl Step for ExtractRegion {
    // a white lie. It's ExtractRegions that sets this tag.
    // But validation happens before the expansion of Transformations
    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        let regions = vec![RegionDefinition {
            source: self.source,
            start: self.start,
            length: self.len,
        }];
        super::validate_regions(&regions, input_def)?;
        Ok(())
    }

    fn apply(
        &mut self,
        _block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        panic!(
            "ExtractRegion is only a configuration step. It is supposed to be replaced by ExtractRegions when the Transformations are expandend"
        );
    }
}

///Extract regions, that is by (target|source, 0-based start, length)
///defined triplets, joined with (possibly empty) separator.
#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegions {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,

    pub label: String,

    #[serde(
        deserialize_with = "bstring_from_string",
        default = "super::default_name_separator"
    )]
    pub region_separator: BString,
}

impl Step for ExtractRegions {
    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        super::validate_regions(&self.regions, input_def)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        //todo: handling if the read is shorter than the regions
        //todo: add test case if read is shorter than the regions
        if block.tags.is_none() {
            block.tags = Some(HashMap::new());
        }
        let mut out = Vec::new();
        for ii in 0..block.len() {
            let extracted = extract_regions(ii, &block, &self.regions);
            let mut h: Vec<Hit> = Vec::new();
            for (region, seq) in self.regions.iter().zip(extracted) {
                if !seq.is_empty() {
                    h.push(Hit {
                        location: Some(HitRegion {
                            target: region.source,
                            start: region.start,
                            len: region.length,
                        }),
                        sequence: seq,
                    });
                }
            }
            if h.is_empty() {
                //if no region was extracted, we do not store a hit
                out.push(TagValue::Missing);
            } else {
                out.push(TagValue::Sequence(Hits::new_multiple(h)));
            }
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label.to_string(), out);

        (block, true)
    }
}

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///Does work with `ExtractRegions` and multiple regions.
///
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInSequence {
    label: String,
    #[serde(default)]
    ignore_missing: bool,
}

impl Step for StoreTagInSequence {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    #[allow(clippy::cast_precision_loss)]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        #[derive(Eq, PartialEq, Debug)]
        enum WhatHappend {
            SameSize,
            Smaller,
            Larger,
        }

        let mut what_happend = Vec::new();

        block.apply_mut_with_tag(&self.label, |read1, read2, index1, index2, tag_val| {
            if let Some(hit) = tag_val.as_sequence() {
                let mut what_happend_here = Vec::new();
                for region in &hit.0 {
                    let location = region
                        .location
                        .as_ref();
                    match location {
                        None => {
                            if self.ignore_missing {
                                //if we ignore missing locations, we just skip this region
                            } else {
                                panic!("StoreTagInSequence only works on regions with location data. Might have been lost on subsequent sequence editing transformations? Region: {region:?}. If you're ok with not sotring those, set ignore_missing=true");
                            }
                        }

                        Some(location) => {

                        let read: &mut crate::io::WrappedFastQReadMut = match location.target {
                            Target::Read1 => read1,
                            Target::Read2 => read2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index1 => index1
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index2 => index2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                        };
                        let seq = read.seq();
                        let mut new_seq: Vec<u8> = Vec::new();
                        new_seq.extend_from_slice(&seq[..location.start]);
                        new_seq.extend_from_slice(&region.sequence);
                        new_seq.extend_from_slice(&seq[location.start + location.len..]);

                        let mut new_qual: Vec<u8> = Vec::new();
                        new_qual.extend_from_slice(&read.qual()[..location.start]);
                        if region.sequence.len() == location.len {
                            //if the sequence is the same length as the location excised, we can just copy the quality
                            new_qual.extend_from_slice(
                                &read.qual()[location.start..location.start + location.len],
                            );
                            what_happend_here.push(WhatHappend::SameSize);
                        } else {
                            //otherwise, we need replace it with the average quality, repeated
                            let avg_qual = if location.is_empty() {
                                b'B'
                            } else {
                                let sum_qual = read.qual()
                                    [location.start..location.start + location.len]
                                    .iter()
                                    .map(|&x| u32::from(x))
                                    .sum::<u32>() ;
                                let avg_qual = f64::from(sum_qual) / location.len as f64;
                                avg_qual.round() as u8
                            };
                            new_qual.extend_from_slice(&vec![avg_qual; region.sequence.len()]);
                                if region.sequence.len() < location.len {
                                    what_happend_here.push(WhatHappend::Smaller);
                                } else {
                                    what_happend_here.push(WhatHappend::Larger);
                                }
                        }
                        new_qual.extend_from_slice(&read.qual()[location.start + location.len..]);

                        read.replace_seq(new_seq, new_qual);
                        }
                    }
                }
                what_happend.push(Some(what_happend_here));
            } else {
                what_happend.push(None);
            }
        });

        filter_tag_locations_all_targets(
            &mut block,
            |_location: &HitRegion, pos: usize| -> NewLocation {
                let what_happend_here = &what_happend[pos];
                match what_happend_here {
                    None => NewLocation::Keep,
                    Some(what_happend_here) => {
                        if what_happend_here
                            .iter()
                            .all(|x| *x == WhatHappend::SameSize)
                        {
                            NewLocation::Keep
                        } else {
                            //now the fun part. TODO
                            //Also todo: test cases
                            //for now, I'll just filter them
                            NewLocation::Remove
                        }
                    }
                }
            },
        );

        (block, true)
    }
}

fn default_comment_separator() -> u8 {
    b'|'
}

fn default_comment_insert_char() -> u8 {
    b' '
}

/// Store currently present tags as comments on read names.
/// Comments are key=value pairs, separated by `comment_separator`
/// which defaults to '|'.
/// They get inserted at the first `comment_insert_char`,
/// which defaults to space. The `comment_insert_char` basically moves
/// to the right.
///
/// That means a read name like
/// @ERR12828869.501 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
/// becomes
/// @ERR12828869.501|key=value|key2=value2 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
///
/// This way, your added tags will survive STAR alignment.
/// (STAR always cuts at the first space, and by default also on /)
///
/// (If the `comment_insert_char` is not present, we simply add at the right)
///
///
/// Be default, comments are only placed on Read1.
/// If you need them somewhere else, or an all reads, change the target (to "All")
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInComment {
    label: String,
    #[serde(default = "default_target_read1")]
    target: TargetPlusAll,

    #[serde(default = "default_comment_separator")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_separator: u8,
    #[serde(default = "default_comment_insert_char")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_insert_char: u8,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,
}

/// Format a numeric value for use in read comments, truncating floats to 4 decimal places
fn format_numeric_for_comment(value: f64) -> String {
    // Check if the value is effectively an integer
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{:.4}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn store_tag_in_comment(
    read: &mut crate::io::WrappedFastQReadMut,
    label: &[u8],
    tag_value: &[u8],
    comment_separator: u8,
    comment_insert_char: u8,
) {
    let name = read.name();
    assert!(
        !tag_value.iter().any(|x| *x == comment_separator),
        "Tag value for {} contains the comment separator '{}'. This would break the read name. Please change the tag value or the comment separator.",
        std::str::from_utf8(label).unwrap_or("utf-8 error"),
        comment_separator as char
    );
    let insert_pos = read
        .name()
        .iter()
        .position(|&x| x == comment_insert_char)
        .unwrap_or(read.name().len());

    let mut new_name =
        Vec::with_capacity(read.name().len() + 1 + label.len() + 1 + tag_value.len());
    new_name.extend_from_slice(&name[..insert_pos]);
    new_name.push(comment_separator);
    new_name.extend_from_slice(label);
    new_name.push(b'=');
    new_name.extend_from_slice(tag_value);
    new_name.extend_from_slice(&name[insert_pos..]);

    read.replace_name(new_name);
}

impl Step for StoreTagInComment {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)?;

        match self.target {
            TargetPlusAll::Read1 => {
                if let Some(output) = output_def {
                    if !output.output_read1 {
                        bail!(
                            "StoreTagInComment is configured to write comments to Read1, but the output does not contain Read1."
                        );
                    }
                }
            }
            TargetPlusAll::Read2 => {
                if let Some(output) = output_def {
                    if !output.output_read2 {
                        bail!(
                            "StoreTagInComment is configured to write comments to Read2, but the output does not contain Read2."
                        );
                    }
                }
            }
            TargetPlusAll::Index1 => {
                if let Some(output) = output_def {
                    if !output.output_index1 {
                        bail!(
                            "StoreTagInComment is configured to write comments to Index1, but the output does not contain Index1."
                        );
                    }
                }
            }
            TargetPlusAll::Index2 => {
                if let Some(output) = output_def {
                    if !output.output_index2 {
                        bail!(
                            "StoreTagInComment is configured to write comments to Index2, but the output does not contain Index2."
                        );
                    }
                }
            }
            TargetPlusAll::All => {}
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_with_tag(
            self.target,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, tag_val: &TagValue| {
                let tag_value: Vec<u8> = match tag_val {
                    TagValue::Sequence(hits) => hits.joined_sequence(Some(&self.region_separator)),
                    TagValue::Numeric(n) => format_numeric_for_comment(*n).into_bytes(),
                    TagValue::Missing => Vec::new(),
                };

                store_tag_in_comment(
                    read,
                    self.label.as_bytes(),
                    &tag_value,
                    self.comment_separator,
                    self.comment_insert_char,
                );
            },
        );

        (block, true)
    }
}

/// Store currently present tag locations as
/// {tag}_location=target:start-end,target:start-end
///
/// (Aligners often keep only the read name).
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTaglocationInComment {
    label: String,
    #[serde(default = "default_target_read1")]
    target: TargetPlusAll,

    #[serde(default = "default_comment_separator")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_separator: u8,
    #[serde(default = "default_comment_insert_char")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_insert_char: u8,
}

impl Step for StoreTaglocationInComment {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let label = format!("{}_location", self.label);
        apply_in_place_wrapped_with_tag(
            self.target,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, tag_val: &TagValue| {
                let mut seq: Vec<u8> = Vec::new();
                if let Some(hits) = tag_val.as_sequence() {
                    let mut first = true;
                    for hit in &hits.0 {
                        if let Some(location) = hit.location.as_ref() {
                            if !first {
                                seq.push(b',');
                            }
                            first = false;
                            seq.extend_from_slice(
                                format!(
                                    "{}:{}-{}",
                                    location.target,
                                    location.start,
                                    location.start + location.len
                                )
                                .as_bytes(),
                            );
                        }
                    }
                }
                store_tag_in_comment(
                    read,
                    label.as_bytes(),
                    &seq,
                    self.comment_separator,
                    self.comment_insert_char,
                );
            },
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractLength {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractLength {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            #[allow(clippy::cast_precision_loss)]
            |read| read.seq().len() as f64,
            #[allow(clippy::cast_precision_loss)]
            |read1, read2, index1, index2| {
                let mut total_length = read1.seq().len();
                if let Some(read2) = read2 {
                    total_length += read2.seq().len();
                }
                if let Some(index1) = index1 {
                    total_length += index1.seq().len();
                }
                if let Some(index2) = index2 {
                    total_length += index2.seq().len();
                }
                total_length as f64
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractMeanQuality {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractMeanQuality {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let quality_scores = read.qual();
                if quality_scores.is_empty() {
                    0.0
                } else {
                    let sum: u32 = quality_scores.iter().map(|&q| u32::from(q)).sum();
                    #[allow(clippy::cast_precision_loss)]
                    {
                        f64::from(sum) / quality_scores.len() as f64
                    }
                }
            },
            |read1, read2, index1, index2| {
                let mut total_sum = 0u64;
                let mut total_length = 0usize;

                // Process read1
                let quality_scores = read1.qual();
                total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                total_length += quality_scores.len();

                // Process read2 if present
                if let Some(read2) = read2 {
                    let quality_scores = read2.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let quality_scores = index1.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let quality_scores = index2.qual();
                    total_sum += quality_scores.iter().map(|&q| u64::from(q)).sum::<u64>();
                    total_length += quality_scores.len();
                }

                if total_length == 0 {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        total_sum as f64 / total_length as f64
                    }
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractGCContent {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractGCContent {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        fn gc_count(sequence: &[u8]) -> usize {
            sequence
                .iter()
                .filter(|&&base| base == b'G' || base == b'C' || base == b'g' || base == b'c')
                .count()
        }
        fn non_n_count(sequence: &[u8]) -> usize {
            sequence
                .iter()
                .filter(|&&base| base != b'N' && base != b'n')
                .count()
        }

        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let sequence = read.seq();
                if sequence.is_empty() {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        (gc_count(sequence) as f64 / non_n_count(sequence) as f64) * 100.0
                    }
                }
            },
            |read1, read2, index1, index2| {
                let mut total_gc_count = 0usize;
                let mut total_length = 0usize;

                for seq in Some(read1)
                    .into_iter()
                    .chain(read2.into_iter())
                    .chain(index1.into_iter())
                    .chain(index2.into_iter())
                {
                    let sequence = seq.seq();
                    total_gc_count += gc_count(sequence);
                    total_length += non_n_count(sequence);
                }

                if total_length == 0 {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        (total_gc_count as f64 / total_length as f64) * 100.0
                    }
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractNCount {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractNCount {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let sequence = read.seq();
                #[allow(clippy::cast_precision_loss)]
                {
                    sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count() as f64
                }
            },
            |read1, read2, index1, index2| {
                let mut total_n_count = 0usize;

                // Process read1
                let sequence = read1.seq();
                total_n_count += sequence
                    .iter()
                    .filter(|&&base| base == b'N' || base == b'n')
                    .count();

                // Process read2 if present
                if let Some(read2) = read2 {
                    let sequence = read2.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let sequence = index1.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let sequence = index2.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                #[allow(clippy::cast_precision_loss)]
                {
                    total_n_count as f64
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractLowComplexity {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractLowComplexity {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                // Calculate the number of transitions
                let mut transitions = 0;
                let seq = read.seq();
                if seq.len() <= 1 {
                    return 0.0;
                }
                for ii in 0..seq.len() - 1 {
                    if seq[ii] != seq[ii + 1] {
                        transitions += 1;
                    }
                }
                transitions as f64 / (seq.len() - 1) as f64
            },
            |read1, read2, index1, index2| {
                let mut total_transitions = 0usize;
                let mut total_positions = 0usize;

                // Process read1
                let seq = read1.seq();
                if seq.len() > 1 {
                    for ii in 0..seq.len() - 1 {
                        if seq[ii] != seq[ii + 1] {
                            total_transitions += 1;
                        }
                    }
                    total_positions += seq.len() - 1;
                }

                // Process read2 if present
                if let Some(read2) = read2 {
                    let seq = read2.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let seq = index1.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let seq = index2.seq();
                    if seq.len() > 1 {
                        for ii in 0..seq.len() - 1 {
                            if seq[ii] != seq[ii + 1] {
                                total_transitions += 1;
                            }
                        }
                        total_positions += seq.len() - 1;
                    }
                }

                if total_positions == 0 {
                    0.0
                } else {
                    total_transitions as f64 / total_positions as f64
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractQualifiedBases {
    pub label: String,
    pub target: TargetPlusAll,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
}

impl Step for ExtractQualifiedBases {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let qual = read.qual();
                let sum: usize = qual
                    .iter()
                    .map(|x| usize::from(*x >= self.min_quality))
                    .sum();
                let pct = sum as f64 / qual.len() as f64;
                pct
            },
            |read1, read2, index1, index2| {
                let mut sum: usize = 0;
                let mut len = 0;
                for read in Some(read1)
                    .into_iter()
                    .chain(read2.into_iter())
                    .chain(index1.into_iter())
                    .chain(index2.into_iter())
                {
                    let qual = read.qual();
                    sum += qual
                        .iter()
                        .map(|x| usize::from(*x >= self.min_quality))
                        .sum::<usize>();
                    len += qual.len();
                }
                let pct = sum as f64 / len as f64;
                pct
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RemoveTag {
    label: String,
}

impl Step for RemoveTag {
    fn removes_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }
        (block, true)
    }
}

#[derive(eserde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreTagsInTable {
    table_filename: String,
    #[serde(default)]
    compression: crate::config::FileFormat,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,

    #[serde(skip)]
    full_output_path: Option<PathBuf>,
    #[serde(skip)]
    output_handle:
        Option<Box<csv::Writer<crate::output::HashedAndCompressedWriter<'static, std::fs::File>>>>,
    #[serde(skip)]
    tags: Option<Vec<String>>,
}

impl std::fmt::Debug for StoreTagsInTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreTagsInTable")
            .field("table_filename", &self.table_filename)
            .field("compression", &self.compression)
            .field("region_separator", &self.region_separator)
            .field("tags", &self.tags)
            .finish_non_exhaustive()
    }
}

impl Clone for StoreTagsInTable {
    fn clone(&self) -> Self {
        Self {
            table_filename: self.table_filename.clone(),
            compression: self.compression,
            region_separator: self.region_separator.clone(),
            full_output_path: self.full_output_path.clone(),
            output_handle: None,
            tags: None,
        }
    }
}

impl Step for StoreTagsInTable {
    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transform_index: usize,
    ) -> Result<()> {
        if matches!(self.compression, crate::config::FileFormat::None) {
            bail!(
                "StoreTagsInTable doesn't support 'None' for 'no output'. Use 'raw' to get uncompressed data."
            );
        }
        let mut tags_set_before = HashSet::new();
        for trafo in &all_transforms[..this_transform_index] {
            if let Some(tag) = trafo.sets_tag() {
                tags_set_before.insert(tag);
            }
            if let Some(tag) = trafo.removes_tag() {
                tags_set_before.remove(&tag);
            }
        }
        if tags_set_before.is_empty() {
            bail!(
                "StoreTagsInTable needs at least one tag to be set before it in the transformation chain."
            );
        }
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &super::InputInfo,
        _output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        self.full_output_path = Some(output_directory.join(&self.table_filename));

        Ok(None)
    }

    fn needs_serial(&self) -> bool {
        true
    }

    fn transmits_premature_termination(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        if let Some(tags) = block.tags.as_mut() {
            if self.tags.is_none() {
                let file_handle = ex::fs::File::create(self.full_output_path.as_ref().unwrap())
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to open table output file: {:?}: {err:?}",
                            self.full_output_path.as_ref().unwrap()
                        )
                    });
                let buffered_writer = crate::output::HashedAndCompressedWriter::new(
                    file_handle.into_inner(),
                    self.compression,
                    false,
                    false,
                    None, // compression_level not exposed for StoreTagsInTable yet
                )
                .expect("Failed to open table output file");
                let writer = csv::WriterBuilder::new()
                    .delimiter(b'\t')
                    .from_writer(buffered_writer);
                self.output_handle = Some(Box::new(writer));

                self.tags = Some(
                    // that's the order we're going to keep
                    {
                        let mut tags = tags.keys().cloned().collect::<Vec<String>>();
                        tags.sort();
                        tags
                    },
                );
                let mut header = vec!["ReadName"];
                for tag in self.tags.as_ref().unwrap() {
                    header.push(tag);
                }
                self.output_handle
                    .as_mut()
                    .unwrap()
                    .write_record(&header)
                    .expect("Failed to write header to table");
            }
            let mut ii = 0;
            let mut iter = block.read1.get_pseudo_iter();
            while let Some(read) = iter.pseudo_next() {
                let mut record = vec![read.name_without_comment().to_vec()];
                for tag in self.tags.as_ref().unwrap() {
                    record.push(match &(tags.get(tag).unwrap()[ii]) {
                        TagValue::Sequence(v) => v.joined_sequence(Some(&self.region_separator)),
                        TagValue::Numeric(n) => n.to_string().into_bytes(),
                        TagValue::Missing => Vec::new(),
                    });
                }
                ii += 1;
                self.output_handle
                    .as_mut()
                    .unwrap()
                    .write_record(record)
                    .expect("Failed to write record to table");
            }
        }

        (block, true)
    }
    fn finalize(
        &mut self,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        self.output_handle
            .take()
            .unwrap() //since we fail in validation if there are no tags, we always have an output
            //handle
            .flush()
            .expect("Failed final csv flush");
        Ok(None)
    }
}

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct QuantifyTag {
    pub infix: String,
    pub label: String,

    #[serde(skip)]
    pub collector: HashMap<Vec<u8>, usize>,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,
}

impl Step for QuantifyTag {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let collector = &mut self.collector;
        let hits = block
            .tags
            .as_ref()
            .expect("No tags in block: bug")
            .get(&self.label)
            .expect("Tag not found. Should have been caught in validation");
        for tag_val in hits {
            if let Some(hit) = tag_val.as_sequence() {
                *collector
                    .entry(hit.joined_sequence(Some(&self.region_separator)))
                    .or_insert(0) += 1;
            }
        }
        (block, true)
    }

    fn finalize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        use std::io::Write;
        let infix = &self.infix;
        let report_file = std::fs::File::create(
            output_directory.join(format!("{output_prefix}_{infix}.qr.json")),
        )?;
        let mut bufwriter = BufWriter::new(report_file);
        let mut str_collector: Vec<(String, usize)> = self
            .collector
            .iter()
            .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
            .collect();
        //sort by count descending, then alphabetically by string
        str_collector.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });
        // we want something that keeps the order
        let str_collector: indexmap::IndexMap<String, usize> = str_collector.into_iter().collect();
        let json = serde_json::to_string_pretty(&str_collector)?;
        bufwriter.write_all(json.as_bytes())?;
        Ok(None)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractRegionsOfLowQuality {
    pub target: Target,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
    pub label: String,
}

impl Step for ExtractRegionsOfLowQuality {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(
            self.target,
            &self.label,
            |read| {
                let quality_scores = read.qual();
                let mut regions = Vec::new();
                let mut in_low_quality_region = false;
                let mut region_start = 0;

                for (pos, &qual) in quality_scores.iter().enumerate() {
                    let is_low_quality = qual < self.min_quality;

                    if is_low_quality && !in_low_quality_region {
                        // Start of a new low quality region
                        in_low_quality_region = true;
                        region_start = pos;
                    } else if !is_low_quality && in_low_quality_region {
                        // End of low quality region
                        in_low_quality_region = false;
                        let region_len = pos - region_start;
                        if region_len > 0 {
                            regions.push(crate::dna::Hit {
                                location: Some(HitRegion {
                                    target: self.target,
                                    start: region_start,
                                    len: region_len,
                                }),
                                sequence: read.seq()[region_start..pos].into(),
                            });
                        }
                    }
                }

                // Handle case where sequence ends with low quality region
                if in_low_quality_region {
                    let region_len = quality_scores.len() - region_start;
                    if region_len > 0 {
                        regions.push(crate::dna::Hit {
                            location: Some(HitRegion {
                                target: self.target,
                                start: region_start,
                                len: region_len,
                            }),
                            sequence: read.seq()[region_start..].into(),
                        });
                    }
                }

                if regions.is_empty() {
                    None
                } else {
                    Some(crate::dna::Hits::new_multiple(regions))
                }
            },
            &mut block,
        );

        (block, true)
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReplaceTagWithLetter {
    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    #[serde(default = "default_replacement_letter")]
    pub letter: u8,
}

fn default_replacement_letter() -> u8 {
    b'N'
}

impl Step for ReplaceTagWithLetter {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn tag_requires_location(&self) -> bool {
        true
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        block.apply_mut_with_tag(&self.label, |read1, read2, index1, index2, tag_val| {
            if let Some(hit) = tag_val.as_sequence() {
                for region in &hit.0 {
                    if let Some(location) = &region.location {
                        let read: &mut crate::io::WrappedFastQReadMut = match location.target {
                            Target::Read1 => read1,
                            Target::Read2 => read2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index1 => index1
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                            Target::Index2 => index2
                                .as_mut()
                                .expect("Input def and transformation def mismatch"),
                        };

                        // Replace the sequence bases in the specified region with the replacement letter
                        let seq = read.seq_mut();
                        for i in location.start..location.start + location.len {
                            if i < seq.len() {
                                seq[i] = self.letter;
                            }
                        }
                    }
                }
            }
        });

        // Remove the consumed tag after processing
        if let Some(tags) = block.tags.as_mut() {
            tags.remove(&self.label);
        }

        (block, true)
    }
}
