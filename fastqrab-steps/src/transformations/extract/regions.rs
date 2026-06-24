use std::{cell::RefCell, collections::HashSet, rc::Rc};

use super::super::{
    PartialRegionDefinition, RegionAnchor, RegionDefinition, extract_from_sequence,
};
use crate::transformations::{prelude::*, read_name_canonical_prefix};
use stringpod::{Lifted, RegionLift};

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
                    if let Some(source) = &inner.source.as_ref().and_then(|x| x.as_ref_post()) {
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
            // For a Location output, record the single segment the tag lives on so
            // a conditional `Swap` on that segment can forget it. A segment/name
            // source gives it directly; a tag source inherits the source tag's
            // segment.
            let segment = inner
                .source
                .as_ref()
                .and_then(|x| x.as_ref_post())
                .and_then(|resolved| match resolved {
                    crate::config::ResolvedSourceNoAll::Segment(idx) => Some(*idx),
                    crate::config::ResolvedSourceNoAll::Name { segment_index, .. } => {
                        Some(*segment_index)
                    }
                    crate::config::ResolvedSourceNoAll::Tag(tag) => {
                        tags_available.get(tag).and_then(|m| m.segment)
                    }
                });
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(output_tag_type).map(|dt| {
                    match (output_tag_type, segment) {
                        (TagValueType::Location, Some(seg)) => dt.with_segment(seg),
                        _ => dt,
                    }
                }),
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
                let input_col = block
                    .tags
                    .get(tag_label)
                    .expect("Input tag missing, validation bug");
                match input_col {
                    // A location source still points into a read: extract relative
                    // to its *joined* sequence and alias the result back into the
                    // same segment, so the output stays a live location column.
                    TagColumn::Location(_) => {
                        let input_segment = input_col.location_segment();
                        // Each region is extracted relative to the source tag's
                        // joined sequence (all its spans concatenated, no
                        // separator): offset 0 is the first covered base, offsets
                        // walk the covered bases in stored order, and offsets past
                        // the join continue linearly into the read from the last
                        // covered base. Snapshot those covered positions up front
                        // so we can drop the borrow on `block.tags` before
                        // rebuilding a column against the segment. Each source span
                        // is lifted from its birth frame into the segment's current
                        // frame first; a span an intervening edit cut (e.g. a
                        // `TrimAtTag` that cleared it) lifts away and contributes
                        // nothing, leaving an empty row.
                        let covered: Vec<Vec<u32>> = {
                            let locations =
                                input_col.as_locations().expect("matched Location above");
                            let segment = &block.segments[input_segment.as_index()];
                            (0..locations.row_count())
                                .map(|row| {
                                    let (born_generation, born_length) = locations.row_born(row);
                                    let view = segment
                                        .seq_quals
                                        .ops_since(born_generation, row)
                                        .expect("born generation from this pod; row in range");
                                    let mut positions = Vec::new();
                                    for (start, len) in locations.row_regions(row) {
                                        if let Ok(RegionLift::Kept { start, len }) = view
                                            .map_region(start as usize, len as usize, born_length)
                                        {
                                            #[expect(clippy::cast_possible_truncation, reason = "lengths are small enough that u32 is sufficient")]
                                            positions.extend(start as u32..(start + len) as u32);
                                        }
                                    }
                                    positions
                                })
                                .collect()
                        };

                        let mut col = block.location_column_builder(input_segment);
                        for (seq_len, covered) in block.segments[input_segment.as_index()]
                            .seq_quals
                            .iter_seq_lens()
                            .zip(&covered)
                        {
                            // It's all or nothing: a read whose source tag has no
                            // location, or where any region falls outside the read,
                            // yields no hit.
                            let parts: Option<Vec<(u32, u32)>> = (!covered.is_empty())
                                .then(|| {
                                    self.regions
                                        .iter()
                                        .map(|region| {
                                            extract_from_joined(
                                                covered,
                                                seq_len,
                                                region.start,
                                                region.length,
                                                &region.anchor,
                                            )
                                        })
                                        .collect::<Option<Vec<Vec<(u32, u32)>>>>()
                                        .map(|per_region| {
                                            per_region.into_iter().flatten().collect()
                                        })
                                })
                                .flatten();
                            col.push_row(parts.as_deref().unwrap_or(&[]));
                        }
                        let col = col.finish();

                        block
                            .tags
                            .insert(self.out_label.clone(), TagColumn::Location(col));
                    }
                    // A string source is just bytes with no read to alias, so the
                    // regions are sliced out of the string value and the output is
                    // itself a string column.
                    TagColumn::String(_) => {
                        let out: StringColumn = input_col
                            .iter_stringified()
                            .map(|value| {
                                value.and_then(|s| extract_string_regions(&s, &self.regions))
                            })
                            .collect();
                        block
                            .tags
                            .insert(self.out_label.clone(), TagColumn::String(out));
                    }
                    _ => panic!(
                        "ExtractRegions tag source must be a Location or String tag (validation bug)"
                    ),
                }
            }
            // A read name is just bytes with no read to alias into, so each
            // region is sliced out of the name's canonical prefix (everything
            // before `split_character`, matching every other `name:` source) and
            // the output is a String column.
            ResolvedSourceNoAll::Name {
                segment_index,
                split_character,
            } => {
                let out: StringColumn = block.segments[segment_index.as_index()]
                    .names
                    .iter()
                    .map(|name| {
                        let prefix = read_name_canonical_prefix(name, Some(*split_character));
                        extract_string_regions(prefix, &self.regions)
                    })
                    .collect();
                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::String(out));
            }
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
                let col = col.finish();

                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::Location(col));
            }
        }

        Ok((block, true))
    }
}

/// Slice every region out of a string-tag value and concatenate them. The
/// regions index into `s` directly (it has no read to anchor against), reusing
/// the same start/length/anchor maths as a whole-segment extraction. All or
/// nothing: returns `None` if any region falls outside `s`.
fn extract_string_regions(s: &[u8], regions: &[RegionDefinition]) -> Option<BString> {
    let mut out = BString::default();
    for region in regions {
        let (start, len) = extract_from_sequence(
            s.len(),
            0,
            s.len(),
            region.start,
            region.length,
            &region.anchor,
        )?;
        out.extend_from_slice(&s[start as usize..(start + len) as usize]);
    }
    Some(out)
}

/// Extract one region from a source tag's *joined* sequence, expressed as
/// read-relative spans.
///
/// `covered` is the source tag's covered read positions, in stored (join) order
/// — so `covered[i]` is the read byte at joined offset `i`. A region is taken in
/// joined-offset space (`out_start` per `anchor`, `out_length` long), then each
/// offset is mapped back to a read position: offsets within the join use
/// `covered`; offsets before it (`< 0`) or past it (`>= covered.len()`) continue
/// linearly into the read from the first / last covered base. The resulting
/// positions are coalesced into contiguous spans (a gappy source can split one
/// region into several). Returns `None` if any offset falls outside the read
/// (`< 0` or `>= seq_len`), so the caller can drop the whole row.
#[expect(clippy::cast_possible_truncation, reason = "lengths are small enough that isize is sufficient")]
#[expect(clippy::cast_possible_wrap, reason = "lengths are small enough that isize is sufficient, and we're always on 64bit systems")]
fn extract_from_joined(
    covered: &[u32],
    seq_len: usize,
    out_start: isize,
    out_length: usize,
    anchor: &RegionAnchor,
) -> Option<Vec<(u32, u32)>> {
    debug_assert!(
        !covered.is_empty(),
        "caller guards against empty source rows"
    );
    let joined_len = covered.len() as isize;
    let first = covered[0] as isize;
    let last = covered[covered.len() - 1] as isize;
    let seq_len = seq_len as isize;
    // The joined-offset of the region's first base.
    let start = match anchor {
        RegionAnchor::Start => out_start,
        RegionAnchor::End => joined_len + out_start,
    };

    let mut spans: Vec<(u32, u32)> = Vec::new();
    for offset in start..start + out_length as isize {
        let read_pos = if offset < 0 {
            first + offset
        } else if offset < joined_len {
            covered[offset as usize] as isize
        } else {
            last + 1 + (offset - joined_len)
        };
        if read_pos < 0 || read_pos >= seq_len {
            return None;
        }
        let read_pos = read_pos as u32;
        // Coalesce runs of consecutive read positions into one span, preserving
        // join order (so a jump back into the read starts a fresh span).
        match spans.last_mut() {
            Some(last) if last.0 + last.1 == read_pos => last.1 += 1,
            _ => spans.push((read_pos, 1)),
        }
    }
    Some(spans)
}
