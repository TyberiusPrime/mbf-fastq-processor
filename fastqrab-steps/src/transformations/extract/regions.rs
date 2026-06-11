use std::{cell::RefCell, collections::HashSet, rc::Rc};

use super::super::{PartialRegionDefinition, RegionDefinition, extract_from_sequence};
use crate::transformations::prelude::*;

/// Extract regions by coordinates
/// that is by (segment|source, 0-based start, length)
/// defined triplets, joined with (possibly empty) separator.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[expect(clippy::struct_field_names, reason = "Step != the actual data")]
pub struct Regions {
    #[tpd(nested)]
    pub regions: Vec<RegionDefinition>, //validated to be non_empty in transformations::validate_regions
    ///
    /// Source for extraction - segment name, "tag:name" for tag source, or "name:segment" for read name source
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "segment")]
    pub source: ResolvedSourceNoAll,

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
        self.source.validate_segment(parent);
        // if let Some(regions) = self.regions.value.as_mut() {
        //     for region in regions.iter_mut() {
        //         if let Some(region_def) = region.value.as_mut() {
        //             //region_def.source.validate_segment(parent);
        //             if region_def.can_concrete() {
        //                 region.state = TomlValueState::Ok;
        //             }
        //         } // cov:excl-line
        //     }
        //     if regions.iter().all(TomlValue::is_ok) {
        //         self.regions.state = TomlValueState::Ok;
        //     }
        // } // cov:excl-line
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
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = vec![];
            let mut seen = HashSet::new();
            let mut all_location = true;
            let mut any_tags = false;
            let all_segments = if let Some(regions) = inner.regions.as_mut() {
                let mut all_segments = true;
                for tv_region in regions.iter_mut() {
                    let source = &inner
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
                                    if !matches!(
                                        provided_tag_types.tag_type,
                                        TagValueType::Location
                                    ) {
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
                all_segments
            } else {
                false
            };
            let output_tag_type = if (any_tags && all_location) || all_segments {
                TagValueType::Location
            } else {
                TagValueType::String
            };
            inner.output_tag_type = Some(output_tag_type);
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(output_tag_type),
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Regions {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        match &self.source {
            ResolvedSourceNoAll::Tag(tag_label) => {
                todo!();
                // let mut out = Vec::with_capacity(block.segments[0].len());
                // for ii in 0..block.len() {
                //     let mut h = BString::default();
                //     let extracted = extract_regions(ii, &block, &self.regions);
                //     for (seq, _coords) in extracted.into_iter().flatten() {
                //         h.push_str(&seq);
                //     }
                //     out.push(Some(h));
                // }
                // block
                //     .tags
                //     .insert(self.out_label.clone(), TagColumn::String(out));
            }
            ResolvedSourceNoAll::Name {
                segment_index,
                split_character,
            } => todo!(),
            ResolvedSourceNoAll::Segment(segment_index) => {
                // Via `location_column_builder` so the column records its source
                // segment (see `TagColumn::location_segment`).
                let mut col = block.location_column_builder(*segment_index);
                for seq_len in block.segments[segment_index.as_index()]
                    .seq_quals
                    .iter_seq_lens()
                {
                    let parts: Vec<_> = self
                        .regions
                        .iter()
                        .filter_map(|region| {
                            extract_from_sequence(
                                seq_len,
                                0,
                                seq_len,
                                region.start,
                                region.length,
                                &region.anchor,
                            )
                        })
                        .collect();
                    col.push_row(&parts);
                }
                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::Location(col.finish()));
            }
        }

        Ok((block, true))
    }
}
