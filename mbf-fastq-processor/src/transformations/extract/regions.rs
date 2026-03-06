#![allow(clippy::unnecessary_wraps)]
use std::{cell::RefCell, collections::HashSet, rc::Rc};

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

    pub out_label: TagLabel,

    #[tpd(skip)]
    #[schemars(skip)]
    pub output_tag_type: TagValueType,
}

impl VerifyIn<PartialConfig> for PartialRegions {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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
            if regions.iter().all(TomlValue::is_ok) {
                self.regions.state = TomlValueState::Ok;
            }
        }
        self.regions.verify(|regions| {
            if regions.is_empty() {
                Err(ValidationFailure::new(
                    "Must contain at least one region definition",
                    None,
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialRegions> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        let mut used_tags = vec![];
        let mut seen = HashSet::new();
        let mut all_location = true;
        let mut any_tags = false;
        let regions = inner.regions.as_mut().expect("Parent was ok?");
        let mut all_segments = true;
        for tv_region in regions.iter_mut() {
            let region = tv_region.as_ref().expect("Parent was ok?");
            let source = region
                .source
                .as_ref()
                .expect("parent was ok")
                .as_ref_post()
                .expect("Not PostVerify");

            if !matches!(source, crate::config::ResolvedSourceNoAll::Segment(_)) {
                all_segments = false;
            }
            if let Some(source_tags) = source.get_tags() {
                any_tags = true;
                let toml_source =
                    Rc::new(RefCell::new((&mut tv_region.state, &mut tv_region.help)));
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
                        used_tags.push(Some(UsedTag {
                            name: entry.0,
                            accepted_tag_types: entry.1,
                            toml_source: toml_source.clone(),
                            further_help: None,
                        }));
                    }
                }
            }
        }
        let output_tag_type = if (any_tags && all_location) || all_segments {
            TagValueType::Location
        } else {
            TagValueType::String
        };
        inner.output_tag_type = Some(output_tag_type);

        TagUsageInfo {
            declared_tag: inner.out_label.to_declared_tag(output_tag_type),
            used_tags,
            ..Default::default()
        }
    }
}

impl Step for Regions {
    // fn uses_tags(
    //     &self,
    //     tags_available: &IndexMap<TagLabel, TagMetadata>,
    // ) -> Option<Vec<(String, &[TagValueType])>> {
    //     let mut tags = Vec::new();
    //     let mut seen = HashSet::new();
    //     let mut all_location = true;
    //     let mut any_tags = false;
    //     for region in &self.regions {
    //         if let Some(source_tags) = region.source.get_tags() {
    //             any_tags = true;
    //             for entry in source_tags {
    //                 if seen.insert(entry.0.clone()) {
    //                     //only add unseen tags
    //                     if let Some(provided_tag_types) = tags_available.get(&entry.0) {
    //                         if !matches!(provided_tag_types.tag_type, TagValueType::Location) {
    //                             all_location = false;
    //                         }
    //                     } else {
    //                         all_location = false;
    //                     }
    //                     tags.push(entry);
    //                 }
    //             }
    //         }
    //     }
    //     let all_segments = self
    //         .regions
    //         .iter()
    //         .all(|x| matches!(x.source, crate::config::ResolvedSourceNoAll::Segment(_)));
    //     if (any_tags && all_location) || all_segments {
    //         self.output_tag_type
    //             .set(TagValueType::Location)
    //             .expect("can't have been set yet");
    //     } else {
    //         self.output_tag_type
    //             .set(TagValueType::String)
    //             .expect("can't have been set yet");
    //     }
    //
    //     if tags.is_empty() { None } else { Some(tags) }
    // }

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
                self.output_tag_type,
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
