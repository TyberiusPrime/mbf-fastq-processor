use crate::transformations::prelude::*;
use fastqrab_dna::dna::reverse_complement_iupac;
use stringpod::Lifted;

/// Reverse complement a read
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ReverseComplement {
    #[tpd(alias = "segment")]
    #[tpd(alias = "source")]
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    target: ResolvedSourceAll,

    #[tpd(alias = "if_label")]
    if_tag: Option<ConditionalTagLabel>,
}

impl VerifyIn<PartialConfig> for PartialReverseComplement {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        self.target
            .deny_name("ReverseComplement does not support name-based targeting");
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialReverseComplement> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = vec![inner.if_tag.to_used_tag(&[])];
            used_tags.extend(inner.target.to_used_tags());
            Some(TagUsageInfo {
                used_tags,
                must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ReverseComplement {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let condition = self
            .if_tag
            .as_ref()
            .map(|tag| get_bool_vec_from_tag(&block, tag));
        match &self.target {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        read.reverse_complement_iupac();
                    },
                    condition.as_deref(),
                );
                // The in-place RC reverses byte order, permuting positions. Record
                // a `reflect` on each affected segment's edit log so a still-attached
                // tag's POSITION lifts to its mirrored span (its captured bytes stay
                // as-is — content only changes when a step targets the tag itself).
                for segment in block.iter_matching_segments_mut(*segment_index_or_all) {
                    segment.seq_quals.record_reverse(condition.as_deref());
                }
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    match hits {
                        // Reverse-complementing a tag changes the *tag's* contents,
                        // not the read. The row COW-detaches into owned bytes; the
                        // read is only ever modified by an explicit write-back step.
                        TagColumn::Location(col) => {
                            for idx in 0..col.row_count() {
                                if col.row_is_empty(idx) {
                                    continue;
                                }
                                if condition.as_ref().is_some_and(|c| !c[idx]) {
                                    continue;
                                }
                                let new_seq = reverse_complement_iupac(&col.joined_seq(idx, None));
                                let mut new_qual = col.joined_qual(idx, None).to_vec();
                                new_qual.reverse();
                                // The row keeps its existing anchor (the original
                                // references); only the bytes diverge.
                                col.set_row_content(idx, &new_seq, &new_qual);
                            }
                        }

                        // A string tag has no read coordinates or quality — just
                        // reverse-complement its stored bytes.
                        TagColumn::String(bstrings) => {
                            for (idx, slot) in bstrings.iter_mut().enumerate() {
                                if condition.as_ref().is_some_and(|c| !c[idx]) {
                                    continue;
                                }
                                if let Some(value) = slot {
                                    *value = reverse_complement_iupac(value).into();
                                }
                            }
                        }
                        _ => unreachable!(), // cov:excl-line
                    }
                } // cov:excl-line
            }
            ResolvedSourceAll::Name { .. } => unreachable!(), // cov:excl-line
        }

        Ok((block, true))
    }
}
