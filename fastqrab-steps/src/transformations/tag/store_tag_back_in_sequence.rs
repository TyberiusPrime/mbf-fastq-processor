use crate::transformations::prelude::*;
use crate::transformations::tag::lifted_writeback::{OnLost, WriteAnchor, store_tag_into_segment};

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///This is exactly [`StoreTagInSequence`] with the tag used as both the content
///source and the position (anchor `Replace`): the tag's (possibly modified) bytes
///overwrite the location they came from. An unmodified location tag is already the
///read's own bytes, so writing it back is a no-op; a tag whose content diverged
///(a regex replacement, a reverse-complement or case change) replaces those bytes.
///Its quality travels with it.
///
///If the location is a single span the content may be longer or shorter than what
///it replaces (the read grows or shrinks); if it covers several disjoint regions
///the content must match their total length. The location is lifted through any
///edits applied since it was captured, so the content lands where the tag *now*
///sits in the read. If an intervening edit cut that location away, `on_lost`
///decides what happens (default: leave the read unchanged).
///
///[`StoreTagInSequence`]: super::store_tag_in_sequence::StoreTagInSequence
///
#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct StoreTagBackInSequence {
    in_label: TagLabel,
    #[tpd(default)]
    ignore_missing: bool,
    /// What to do when the tag's location was lost to a later edit.
    #[tpd(default)]
    #[tpd(alias="on_loss")]
    on_lost: OnLost,
}

impl TagUser for PartialTaggedVariant<PartialStoreTagBackInSequence> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location])],
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for StoreTagBackInSequence {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Store-back is StoreTagInSequence with the same tag as both content and
        // position, anchor `Replace`. The position column borrows `block.tags`; the
        // segment borrows the disjoint `block.segments`, so both can be held at once.
        let value = block
            .tags
            .get(&self.in_label)
            .expect("Tag must be present in block");
        let TagColumn::Location(position) = value else {
            panic!(
                "StoreTagBackInSequence requires a Location tag '{}'",
                self.in_label
            );
        };
        let segment_index = position.source_id() as usize;
        store_tag_into_segment(
            &mut block.segments[segment_index],
            position,
            value,
            WriteAnchor::Replace,
            self.on_lost,
            &format!("StoreTagBackInSequence({})", self.in_label)
        )?;

        Ok((block, true))
    }
}
