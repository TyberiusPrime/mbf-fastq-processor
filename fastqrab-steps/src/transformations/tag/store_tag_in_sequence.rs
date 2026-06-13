use crate::transformations::prelude::*;
use crate::transformations::tag::lifted_writeback::{OnLost, WriteAnchor, store_tag_into_segment};

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
/// A location value carries its own (possibly edited) quality with it; a string value has
/// no quality, so its inserted bases get `~` (Phred 93, maximum Sanger quality).
///
/// `in_position_label` must be a location tag — it defines *where* to act:
///
/// - `anchor = "Start"` (alias `"left"`) — inserts **before** the leftmost start of the
///   location (the hit with the smallest `start` coordinate).
/// - `anchor = "End"` (alias `"right"`) — inserts **after** the rightmost end of the
///   location (the hit with the largest `start + len` value).
/// - `anchor = "Replace"` — overwrites the location. A single span (or several regions that
///   lift to a contiguous range) may be replaced by content of any length; several disjoint
///   regions require the content to match their total length, and the content is then laid
///   down byte-for-byte over exactly the covered positions. Any other disjoint case is a
///   runtime error.
///
/// All coordinates are lifted through edits applied since the position tag was captured, and
/// the length change is recorded so later location tags shift correctly.
///
/// If either tag value is `Missing`, the position tag carries no location, or an edit cut the
/// position away (subject to `on_lost`), the read is left unchanged.
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
    /// What to do when the position tag's location was lost to a later edit.
    #[tpd(alias = "on_loss")]
    #[tpd(default)]
    on_lost: OnLost,
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
        // Pick the content (value) and position columns, then hand both to the
        // shared write-back path. The two columns borrow `block.tags`; the segment
        // borrows the disjoint `block.segments`, so all three can be held at once.
        let value = block
            .tags
            .get(&self.in_value_label)
            .expect("value tag must be present");
        let TagColumn::Location(position) = block
            .tags
            .get(&self.in_position_label)
            .expect("position tag must be present")
        else {
            panic!(
                "position tag '{}' must be a Location column",
                self.in_position_label
            );
        };
        let anchor = match self.anchor {
            ReplacementAnchor::Start => WriteAnchor::Start,
            ReplacementAnchor::End => WriteAnchor::End,
            ReplacementAnchor::Replace => WriteAnchor::Replace,
        };
        let segment_index = position.source_id() as usize;
        store_tag_into_segment(
            &mut block.segments[segment_index],
            position,
            value,
            anchor,
            self.on_lost,
            &format!(
                "StoreTagInSequence({}->{})",
                self.in_value_label, self.in_position_label
            ),
        )?;

        Ok((block, true))
    }
}
