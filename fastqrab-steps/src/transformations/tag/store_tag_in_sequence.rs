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
        // Pass 1: from the value + position tag columns, compute each read's
        // (left, right, insert_bytes) splice. Scoped so the borrows of block.tags
        // drop before we rebuild block.segments.
        let (segment_index, edits): (usize, Vec<Option<(usize, usize, Vec<u8>)>>) = {
            let TagColumn::Location(position_col) = block
                .tags
                .get(&self.in_position_label)
                .expect("position tag must be present")
            else {
                panic!("position tag '{}' must be a Location column", self.in_position_label);
            };
            let value_col = block
                .tags
                .get(&self.in_value_label)
                .expect("value tag must be present");

            let mut edits: Vec<Option<(usize, usize, Vec<u8>)>> =
                Vec::with_capacity(position_col.row_count());
            for row in 0..position_col.row_count() {
                if position_col.row_is_empty(row) {
                    edits.push(None);
                    continue;
                }
                let regions: Vec<(usize, usize)> = position_col
                    .row_regions(row)
                    .map(|(s, l)| (s as usize, l as usize))
                    .collect();
                let (left, right) = match self.anchor {
                    ReplacementAnchor::Start => {
                        let start = regions.iter().map(|&(s, _)| s).min().expect("non-empty");
                        (start, start)
                    }
                    ReplacementAnchor::End => {
                        let end = regions.iter().map(|&(s, l)| s + l).max().expect("non-empty");
                        (end, end)
                    }
                    ReplacementAnchor::Replace => {
                        if regions.len() != 1 {
                            anyhow::bail!(
                                "Error processing StoreTagInSequence: Found a multi region location, StoreTagInSequence only works with single-region location"
                            );
                        }
                        let (s, l) = regions[0];
                        (s, s + l)
                    }
                };
                let insert_bytes: Vec<u8> = match value_col {
                    TagColumn::Location(col) => {
                        if col.row_is_empty(row) {
                            edits.push(None);
                            continue;
                        }
                        col.joined_seq(row, None).to_vec()
                    }
                    TagColumn::String(items) => match &items[row] {
                        Some(value) => value.to_vec(),
                        None => {
                            edits.push(None);
                            continue;
                        }
                    },
                    _ => panic!("value tag '{}' must be Location or String", self.in_value_label),
                };
                if insert_bytes.is_empty() {
                    edits.push(None);
                } else {
                    edits.push(Some((left, right, insert_bytes)));
                }
            }
            (position_col.source_id() as usize, edits)
        };

        if edits.iter().all(Option::is_none) {
            return Ok((block, true));
        }

        // Pass 2: rebuild the segment's reads with the splices applied. Inserted
        // bases get '~' (Phred 93) quality.
        let segment = &mut block.segments[segment_index];
        assert_eq!(
            segment.seq_quals.len(),
            edits.len(),
            "tag column and segment row counts must match"
        );
        let mut builder = DualStringPodBuilder::with_capacity(0, edits.len());
        for (ii, read) in segment.seq_quals.iter().enumerate() {
            match &edits[ii] {
                None => builder.push(read.seq, read.qual),
                Some((left, right, insert_bytes)) => {
                    let seq = read.seq;
                    let qual = read.qual;
                    let left = (*left).min(seq.len());
                    let right = (*right).min(seq.len()).max(left);
                    let mut out_seq =
                        Vec::with_capacity(seq.len() - (right - left) + insert_bytes.len());
                    out_seq.extend_from_slice(&seq[..left]);
                    out_seq.extend_from_slice(insert_bytes);
                    out_seq.extend_from_slice(&seq[right..]);
                    let mut out_qual = Vec::with_capacity(out_seq.len());
                    out_qual.extend_from_slice(&qual[..left]);
                    out_qual.extend_from_slice(&vec![b'~'; insert_bytes.len()]);
                    out_qual.extend_from_slice(&qual[right..]);
                    builder.push(&out_seq, &out_qual);
                }
            }
        }
        segment.seq_quals = builder.finish();

        Ok((block, true))
    }
}
