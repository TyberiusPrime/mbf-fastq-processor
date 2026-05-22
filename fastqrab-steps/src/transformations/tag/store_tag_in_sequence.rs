use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
enum ReplacementAnchor {
    #[tpd(alias = "left")]
    Start,
    #[tpd(alias = "right")]
    End,
    Replace,
}

/// Insert a tag's string value into a read sequence at the position defined by another
/// location tag.
///
/// `in_value_label` must be a location or string tag — its byte sequence (via `as_ref()` on
/// the value) is the text inserted into the read.
///
/// `in_position_label` must be a location tag — it defines *where* to insert:
///
/// - `anchor = "Start"` (alias `"left"`) — inserts **before** the leftmost start of the
///   location (the hit with the smallest `start` coordinate).
/// - `anchor = "End"` (alias `"right"`) — inserts **after** the rightmost end of the
///   location (the hit with the largest `start + len` value).
///
/// Quality scores for the inserted bases are set to `~` (Phred 93, maximum Sanger quality).
///
/// After insertion, all location tags on the same segment whose start position is ≥ the
/// insertion point are shifted forward by the number of inserted bases.  Any location that
/// straddles the insertion point (starts before it, ends after it) has its location
/// information removed while its sequence data is preserved.
///
/// If either tag value is `Missing`, or the position tag carries no location information,
/// the read is left unchanged.
///
#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct StoreTagInSequence {
    /// Location or string tag whose sequence to insert into the read.
    in_value_label: TagLabel,
    /// Location tag that defines the insertion position.
    in_position_label: TagLabel,
    /// Where to insert relative to the position tag.
    anchor: ReplacementAnchor,
}

impl TagUser for PartialTaggedVariant<PartialStoreTagInSequence> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_value_label
                        .to_used_tag(&[TagValueType::Location, TagValueType::String]),
                    inner
                        .in_position_label
                        .to_used_tag(&[TagValueType::Location]),
                ],
                must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for StoreTagInSequence {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Per-read record of where an insertion happened (None = no insertion for this read)
        struct InsertInfo {
            segment_idx: SegmentIndex,
            insert_pos_left: usize,
            insert_pos_right: usize,
            insert_len: usize,
        }

        let mut insert_infos: Vec<Option<InsertInfo>> = Vec::with_capacity(block.len());
        // Pass 1: compute (seg_idx, insert_pos, insert_bytes) per read from the tag columns.
        // Scoped so the immutable borrows of block.tags drop before we mutate block.segments.
        // per_read: (seg_idx, pos_left, pos_right, insert_bytes)
        let per_read: Vec<Option<(SegmentIndex, usize, usize, Vec<u8>)>> = {
            let value_col = block
                .tags
                .get(&self.in_value_label)
                .expect("value tag must be present");
            let position_items = match block
                .tags
                .get(&self.in_position_label)
                .expect("position tag must be present")
            {
                TagColumn::Location(items) => items,
                _ => panic!("position tag must be a Location column"),
            };
            let n = position_items.len();
            let mut out = Vec::with_capacity(n);
            for ii in 0..n {
                let position = match &position_items[ii] {
                    None => {
                        out.push(None);
                        continue;
                    }
                    Some(hits) => match self.anchor {
                        ReplacementAnchor::Start => hits
                            .0
                            .iter()
                            .filter_map(|h| h.location.as_ref())
                            .min_by_key(|loc| loc.start)
                            .map(|loc| (loc.start, loc.start, loc.segment_index)),
                        ReplacementAnchor::End => hits
                            .0
                            .iter()
                            .filter_map(|h| h.location.as_ref())
                            .max_by_key(|loc| loc.start + loc.len)
                            .map(|loc| {
                                let end = loc.start + loc.len;
                                (end, end, loc.segment_index)
                            }),
                        ReplacementAnchor::Replace => {
                            let locs: Vec<_> =
                                hits.0.iter().filter_map(|h| h.location.as_ref()).collect();
                            if locs.len() > 1 {
                                anyhow::bail!(
                                    "Error processing StoreTagInSequence: Found a multi region location, StoreTagInSequence only works with single-region location"
                                );
                            }
                            locs.first()
                                .map(|loc| (loc.start, loc.start + loc.len, loc.segment_index))
                        }
                    },
                };
                let Some((pos_left, pos_right, seg_idx)) = position else {
                    out.push(None);
                    continue;
                };
                let insert_bytes: Vec<u8> = match value_col {
                    TagColumn::Location(items) => match &items[ii] {
                        Some(hits) => hits.joined_sequence(None),
                        None => {
                            out.push(None);
                            continue;
                        }
                    },
                    TagColumn::String(items) => match &items[ii] {
                        Some(s) => s.to_vec(),
                        None => {
                            out.push(None);
                            continue;
                        }
                    },
                    _ => panic!("value tag must be Location or String"),
                };
                if insert_bytes.is_empty() {
                    out.push(None);
                } else {
                    out.push(Some((seg_idx, pos_left, pos_right, insert_bytes)));
                }
            }
            out
        };

        // Pass 2: apply sequence insertions.
        for (ii, slot) in per_read.iter().enumerate() {
            if let Some((seg_idx, pos_left, pos_right, insert_bytes)) = slot {
                let (seg_idx, pos_left, pos_right) = (*seg_idx, *pos_left, *pos_right);
                block.segments[seg_idx.as_index()].mutate_read_at(ii, |read| {
                    let seq = read.seq().to_vec();
                    // cov:excl-start
                    assert!(
                        pos_right <= seq.len(),
                        "StoreTagInSequence: insert position {pos_right} exceeds read length \
                        {}. This should have been prevented upstream and is a bug.",
                        seq.len(),
                    );
                    // cov:excl-end
                    let mut new_seq = Vec::with_capacity(seq.len() + insert_bytes.len());
                    new_seq.extend_from_slice(&seq[..pos_left]);
                    new_seq.extend_from_slice(insert_bytes);
                    new_seq.extend_from_slice(&seq[pos_right..]);
                    let qual = read.qual().to_vec();
                    let mut new_qual = Vec::with_capacity(qual.len() + insert_bytes.len());
                    new_qual.extend_from_slice(&qual[..pos_left]);
                    new_qual.extend_from_slice(&vec![b'~'; insert_bytes.len()]);
                    new_qual.extend_from_slice(&qual[pos_right..]);
                    read.replace_seq(&new_seq, &new_qual);
                });
                insert_infos.push(Some(InsertInfo {
                    segment_idx: seg_idx,
                    insert_pos_left: pos_left,
                    insert_pos_right: pos_right,
                    insert_len: insert_bytes.len(),
                }));
            } else {
                insert_infos.push(None);
            }
        }

        // Shift all location tags whose start is >= the insertion point, and
        // invalidate any that straddle it (start before, end after).
        let num_segments = block.segments.len();
        for seg_idx in 0..num_segments {
            let segment_index = SegmentIndex::new(seg_idx);
            block.filter_tag_locations(
                segment_index,
                |location: &HitRegion, read_pos: usize, _seq: &BString, _read_len: usize| {
                    match &insert_infos[read_pos] {
                        Some(info) if info.segment_idx.as_index() == seg_idx => {
                            if location.start >= info.insert_pos_right {
                                // Entirely after the insertion → shift by net delta
                                // (insert_len minus the number of bytes removed, which is
                                // zero for Start/End anchors and replaced_len for Replace)
                                NewLocation::New(HitRegion {
                                    start: location.start + info.insert_len
                                        - (info.insert_pos_right - info.insert_pos_left),
                                    len: location.len,
                                    segment_index: location.segment_index,
                                })
                            } else if location.start + location.len > info.insert_pos_left {
                                // Straddles the insertion point → remove location info
                                NewLocation::Remove
                            } else {
                                // Entirely before the insertion → unchanged
                                NewLocation::Keep
                            }
                        }
                        _ => NewLocation::Keep,
                    }
                },
                None,
            );
        }

        Ok((block, true))
    }
}
