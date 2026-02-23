#![allow(clippy::unnecessary_wraps)]
use std::{collections::HashSet, sync::OnceLock};

//eserde false positives
//
use crate::transformations::prelude::*;

use super::super::{PartialRegionDefinition, RegionDefinition, extract_regions};
use crate::dna::{Hit, HitRegion, TagValue};
use bstr::ByteVec;
use toml_pretty_deser::Visitor;

/// Extract regions by coordinates
/// that is by (segment|source, 0-based start, length)
/// defined triplets, joined with (possibly empty) separator.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct Regions {
    #[tpd(nested)]
    pub regions: Vec<RegionDefinition>, //validated to be non_empty in transformations::validate_regions

    pub out_label: String,

    /* #[serde(deserialize_with = "crate::config::deser::option_bstring_from_string")]
    #[schemars(with = "Option<String>")]
    #[serde(default)]
    pub region_separator: Option<BString>, */
    #[tpd(skip, default)]
    #[schemars(skip)]
    pub output_tag_type: OnceLock<TagValueType>,
}

impl VerifyIn<PartialConfig> for PartialRegions {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(regions) = self.regions.value.as_mut() {
            for region in regions.iter_mut() {
                if let Some(region_def) = region.value.as_mut() {
                    region_def.source.validate_segment(parent);
                    if region_def.can_concrete() {
                        region.state = TomlValueState::Ok;
                    }
                }
            }
            if regions.iter().all(|x| x.is_ok()) {
                self.regions.state = TomlValueState::Ok
            }
        }
        Ok(())
    }
}

impl Step for Regions {
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((
            self.out_label.clone(),
            *self
                .output_tag_type
                .get()
                .expect("Expect tag type to be set at this point"),
        ))
    }

    fn uses_tags(
        &self,
        tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        let mut tags = Vec::new();
        let mut seen = HashSet::new();
        let mut all_location = true;
        let mut any_tags = false;
        for region in &self.regions {
            if let Some(source_tags) = region.source.get_tags() {
                any_tags = true;
                for entry in source_tags {
                    if seen.insert(entry.0.clone()) {
                        //only add unseen tags
                        if let Some(provided_tag_types) = tags_available.get(&entry.0) {
                            if !matches!(provided_tag_types.tag_type, TagValueType::Location) {
                                all_location = false;
                            }
                        } else {
                            all_location = false;
                        }
                        tags.push(entry);
                    }
                }
            }
        }
        let all_segments = self
            .regions
            .iter()
            .all(|x| matches!(x.source, crate::config::ResolvedSourceNoAll::Segment(_)));
        if (any_tags && all_location) || all_segments {
            self.output_tag_type
                .set(TagValueType::Location)
                .expect("can't have been set yet");
        } else {
            self.output_tag_type
                .set(TagValueType::String)
                .expect("can't have been set yet");
        }

        if tags.is_empty() { None } else { Some(tags) }
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut out = Vec::with_capacity(block.segments[0].len());
        for ii in 0..block.len() {
            let extracted = extract_regions(ii, &block, &self.regions);
            if extracted.iter().any(Option::is_none) {
                //if any region could not be extracted, we store Missing
                out.push(TagValue::Missing);
                continue;
            }
            //all segments -> Location.
            if matches!(
                self.output_tag_type
                    .get()
                    .as_ref()
                    .expect("tag type not defined?!",),
                crate::transformations::TagValueType::Location
            ) {
                let mut h: Vec<Hit> = Vec::new();
                for (seq, opt_coords) in extracted.into_iter().flatten() {
                    // eats Nones.
                    if let Some(coords) = opt_coords {
                        h.push(Hit {
                            location: Some(HitRegion {
                                segment_index: coords.segment_index,
                                start: coords.start,
                                len: coords.length,
                            }),
                            sequence: seq,
                        });
                    } else if !seq.is_empty() {
                        unreachable!();
                    }
                }
                if h.is_empty() {
                    //if no region was extracted, we do not store a hit
                    out.push(TagValue::Missing);
                } else {
                    out.push(TagValue::Location(crate::dna::Hits::new_multiple(h)));
                }
            } else {
                let mut h = BString::default();
                for (seq, _segment_index) in extracted.into_iter().flatten() {
                    h.push_str(&seq);
                }
                out.push(TagValue::String(h));
            }
        }

        block.tags.insert(self.out_label.clone(), out);

        Ok((block, true))
    }
}
