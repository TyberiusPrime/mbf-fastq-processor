#![allow(clippy::unnecessary_wraps)] //eserde false positives
//
use crate::transformations::prelude::*;

use std::collections::HashMap;

use super::super::{RegionDefinition, extract_regions};
use crate::dna::{Hit, HitRegion, TagValue};
use serde_valid::Validate;

///Extract regions, that is by (segment|source, 0-based start, length)
///defined triplets, joined with (possibly empty) separator.
#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_field_names)]
pub struct Regions {
    #[validate(min_items = 1)]
    pub regions: Vec<RegionDefinition>,

    pub out_label: String,
    /* #[serde(
        deserialize_with = "bstring_from_string",
        default = "super::super::default_name_separator_bstring"
    )]
    pub region_separator: BString, */
}

impl Step for Regions {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        super::super::validate_regions(&mut self.regions, input_def)
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
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
                            segment_index: region.segment_index.unwrap(),
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
                out.push(TagValue::Location(crate::dna::Hits::new_multiple(h)));
            }
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.out_label.to_string(), out);

        Ok((block, true))
    }
}
