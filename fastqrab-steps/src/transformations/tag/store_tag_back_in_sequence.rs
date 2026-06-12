use crate::transformations::prelude::*;

///Store the tag's 'sequence', probably modified by a previous step,
///back into the reads' sequence.
///
///Only rows whose content has *diverged* from the read (owned rows — produced by
///a regex replacement, a reverse-complement or case change on the tag, etc.) are
///written back; an unmodified location tag is already the read's own bytes, so
///writing it back is a no-op. The spliced content may be longer or shorter than
///the span it replaces, so the read grows or shrinks accordingly; its quality
///travels with the tag.
///
#[derive(Clone, JsonSchema)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct StoreTagBackInSequence {
    in_label: TagLabel,
    #[tpd(default)]
    ignore_missing: bool,
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
        // Collect the per-read splices (owned/divergent rows only), then drop the
        // borrow on `block.tags` before rebuilding the segment's reads.
        let (segment_index, edits) = {
            let TagColumn::Location(col) = block
                .tags
                .get(&self.in_label)
                .expect("Tag must be present in block")
            else {
                panic!("StoreTagBackInSequence requires a Location tag '{}'", self.in_label);
            };
            let mut edits: Vec<Option<(usize, usize, Vec<u8>, Vec<u8>)>> =
                Vec::with_capacity(col.row_count());
            for row in 0..col.row_count() {
                match col.owned_writeback_span(row) {
                    Some((start, len)) => edits.push(Some((
                        start,
                        len,
                        col.joined_seq(row, None).to_vec(),
                        col.joined_qual(row, None).to_vec(),
                    ))),
                    None => edits.push(None),
                }
            }
            (col.source_id() as usize, edits)
        };

        // Nothing diverged → leave the reads untouched.
        if edits.iter().all(Option::is_none) {
            return Ok((block, true));
        }

        // Rebuild the segment's seq+qual with the splices applied. A length change
        // can't live in the read pod's overlay, so we rebuild eagerly (the read's
        // own bytes are copied; spliced rows get the tag's owned bytes).
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
                Some((start, len, new_seq, new_qual)) => {
                    let seq = read.seq;
                    let qual = read.qual;
                    let start = (*start).min(seq.len());
                    let end = (start + *len).min(seq.len());
                    let mut out_seq = Vec::with_capacity(seq.len() - (end - start) + new_seq.len());
                    out_seq.extend_from_slice(&seq[..start]);
                    out_seq.extend_from_slice(new_seq);
                    out_seq.extend_from_slice(&seq[end..]);
                    let mut out_qual = Vec::with_capacity(out_seq.len());
                    out_qual.extend_from_slice(&qual[..start]);
                    out_qual.extend_from_slice(new_qual);
                    out_qual.extend_from_slice(&qual[end..]);
                    builder.push(&out_seq, &out_qual);
                }
            }
        }
        segment.seq_quals = builder.finish();

        Ok((block, true))
    }
}
