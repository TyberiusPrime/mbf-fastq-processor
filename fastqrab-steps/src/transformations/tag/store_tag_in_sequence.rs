use crate::transformations::{RegionAnchor, prelude::*};

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
    /// Where to insert relative to the position tag:
    /// `"Start"` / `"left"` inserts before the leftmost position;
    /// `"End"` / `"right"` inserts after the rightmost end.
    anchor: RegionAnchor,
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
            segment_idx: usize,
            insert_pos: usize,
            insert_len: usize,
        }

        let mut insert_infos: Vec<Option<InsertInfo>> = Vec::with_capacity(block.len());

        block.apply_mut_with_tags(
            &self.in_value_label,
            &self.in_position_label,
            |reads, value_tag, position_tag| {
                // Obtain the bytes to insert from the value tag
                let value_bstr = value_tag.to_bstr();
                let insert_bytes: &[u8] = value_bstr.as_ref();

                if insert_bytes.is_empty() {
                    insert_infos.push(None);
                    return;
                }

                // Find the location that defines the insert position
                let Some(hits) = position_tag.as_sequence() else {
                        insert_infos.push(None);
                        return;
                    };

                // Select the anchor hit region based on anchor direction
                let anchor_region: Option<&HitRegion> = match self.anchor {
                    RegionAnchor::Start => hits
                        .0
                        .iter()
                        .filter_map(|h| h.location.as_ref())
                        .min_by_key(|loc| loc.start),
                    RegionAnchor::End => hits
                        .0
                        .iter()
                        .filter_map(|h| h.location.as_ref())
                        .max_by_key(|loc| loc.start + loc.len),
                };

                let Some(region) = anchor_region else {
                        // Position tag has no location info — skip
                        insert_infos.push(None);
                        return;
                };

                let insert_pos = match self.anchor {
                    RegionAnchor::Start => region.start,
                    RegionAnchor::End => region.start + region.len,
                };
                let seg_idx = region.segment_index.get_index();

                let read = &mut reads[seg_idx];
                let seq = read.seq();

                assert!(insert_pos <= seq.len(),
                        "StoreTagInSequence: insert position {insert_pos} exceeds read length \
                        {} on segment {seg_idx}. THis should have been prevent upstream and is a bug.
                        coordinates are within the read.",
                        seq.len(),
                );

                let mut new_seq = Vec::with_capacity(seq.len() + insert_bytes.len());
                new_seq.extend_from_slice(&seq[..insert_pos]);
                new_seq.extend_from_slice(insert_bytes);
                new_seq.extend_from_slice(&seq[insert_pos..]);

                let qual = read.qual();
                let mut new_qual = Vec::with_capacity(qual.len() + insert_bytes.len());
                new_qual.extend_from_slice(&qual[..insert_pos]);
                new_qual.extend_from_slice(&vec![b'~'; insert_bytes.len()]);
                new_qual.extend_from_slice(&qual[insert_pos..]);

                read.replace_seq(&new_seq, &new_qual);

                insert_infos.push(Some(InsertInfo {
                    segment_idx: seg_idx,
                    insert_pos,
                    insert_len: insert_bytes.len(),
                }));
            },
        );

        // Shift all location tags whose start is >= the insertion point, and
        // invalidate any that straddle it (start before, end after).
        let num_segments = block.segments.len();
        for seg_idx in 0..num_segments {
            let segment_index = SegmentIndex(seg_idx);
            block.filter_tag_locations(
                segment_index,
                |location: &HitRegion, read_pos: usize, _seq: &BString, _read_len: usize| {
                    match &insert_infos[read_pos] {
                        Some(info) if info.segment_idx == seg_idx => {
                            if location.start >= info.insert_pos {
                                // Entirely after the insertion → shift forward
                                NewLocation::New(HitRegion {
                                    start: location.start + info.insert_len,
                                    len: location.len,
                                    segment_index: location.segment_index,
                                })
                            } else if location.start + location.len > info.insert_pos {
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
