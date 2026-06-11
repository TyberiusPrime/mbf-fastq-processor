use crate::transformations::prelude::*;
use fastqrab_dna::dna::reverse_complement_iupac;

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
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    match hits {
                        TagColumn::Location(col) => {
                            let segment = &mut block.segments[col.source_id() as usize];
                            for (idx, read) in segment.seq_quals.iter_mut().enumerate() {
                                if condition.as_ref().is_none_or(|c| c[idx]) {
                                    let new_seq =
                                        reverse_complement_iupac(&col.joined_seq(idx, None));
                                    let new_qual = col.joined_qual(idx, None);
                                    for ((pos, new_base), new_qual) in col
                                        .covered_positions(idx)
                                        .zip(new_seq)
                                        .zip(new_qual.iter().rev())
                                    {
                                        read.seq[pos] = new_base;
                                        read.qual[pos] = *new_qual;
                                    }
                                }
                            }
                        }

                        TagColumn::String(bstrings) => {
                            todo!(
                                "Decide what to do. This seems terribly wrong,\
                            to at one hand mutate the reads (on location tags)
                            and on the other to be manipulating string tags
                            in place.
                                Maybe we should just require teh tag to be an Location

                                "
                            );
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
