#![allow(clippy::unnecessary_wraps)]
use std::{collections::HashSet, sync::OnceLock};

//eserde false positives
//
use crate::transformations::prelude::*;

use super::super::{RegionDefinition, TagValueType, extract_regions};
use crate::dna::{Hit, HitRegion, TagValue};
use bstr::ByteVec;
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

    /* #[serde(deserialize_with = "crate::config::deser::option_bstring_from_string")]
    #[schemars(with = "Option<String>")]
    #[serde(default)]
    pub region_separator: Option<BString>, */
    #[serde(default)]
    #[serde(skip)]
    pub output_tag_type: OnceLock<crate::transformations::TagValueType>,
}

impl Step for Regions {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            *self
                .output_tag_type
                .get()
                .expect("Expect tag type to be set at this point"),
        ))
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        super::super::validate_regions(&mut self.regions, input_def)?;
        /* if self.regions.len() > 1 && self.region_separator.is_none() {
            bail!("When extracting multiple regions, a region_separator must be provided. Can be an empty string.");
        } */
        Ok(())
    }

    fn uses_tags(
        &self,
        tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        let mut tags = Vec::new();
        let mut seen = HashSet::new();
        let mut all_location = true;
        for region in &self.regions {
            if let Some(ref resolved_source) = region.resolved_source
                && let Some(source_tags) = resolved_source.get_tags()
            {
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
        let all_segments = self.regions.iter().all(|x| {
            matches!(
                x.resolved_source.as_ref().expect("Must have been resolved"),
                crate::transformations::ResolvedSource::Segment(_)
            )
        });
        if all_location || all_segments {
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
                        //we got a sequence, but no segment index -> cannot store location
                        h.push(Hit {
                            location: None,
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
            } else {
                let mut h = BString::default();
                for (seq, _segment_index) in extracted.into_iter().flatten() {
                    h.push_str(&seq);
                }
                out.push(TagValue::String(h));
            }
        }

        block.tags.insert(self.out_label.to_string(), out);

        Ok((block, true))
    }
}
