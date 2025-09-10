#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::collections::HashMap;

use crate::{
    config::deser::bstring_from_string,
    dna::{Hit, HitRegion, TagValue},
    Demultiplexed,
};
use anyhow::Result;
use serde_valid::Validate;

use super::super::{extract_regions, RegionDefinition, Step};

///Extract regions, that is by (segment|source, 0-based start, length)
///defined triplets, joined with (possibly empty) separator.
#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)]
pub struct Regions {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,

    pub label: String,

    #[serde(
        deserialize_with = "bstring_from_string",
        default = "super::super::default_name_separator"
    )]
    pub region_separator: BString,
}

impl Step for Regions {
    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        super::super::validate_regions(&mut self.regions, input_def)
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
                            segment: region.source.clone(),
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
                out.push(TagValue::Sequence(crate::dna::Hits::new_multiple(h)));
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
