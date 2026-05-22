use crate::transformations::prelude::*;
use fastqrab_dna::dna::{HitRegion, reverse_complement_iupac};

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
                        read.reverse_complement();
                    },
                    condition.as_deref(),
                );
                let ftl =
                    |location: HitRegion, _pos, seq: &[u8], read_len: usize| -> NewLocation {
                        let new_start = read_len
                            .checked_sub(location.start + location.len)
                            .expect("Start position underflow");
                        let new_seq = reverse_complement_iupac(seq);
                        NewLocation::NewWithSeq(
                            HitRegion {
                                start: new_start,
                                len: location.len,
                                segment_index: location.segment_index,
                            },
                            new_seq.into(),
                        )
                    };
                match segment_index_or_all {
                    SegmentIndexOrAll::All => {
                        for idx in 0..block.segments.len() {
                            block.filter_tag_locations(
                                SegmentIndex::new(idx),
                                ftl,
                                condition.as_deref(),
                            );
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment) => {
                        block.filter_tag_locations(*segment, ftl, condition.as_deref())
                    }
                }
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    match hits {
                        TagColumn::Location(col) => {
                            for slot_idx in 0..col.hits.len() {
                                let nhits = col.hits[slot_idx].len();
                                for hit_idx in 0..nhits {
                                    let hit = col.hits[slot_idx][hit_idx];
                                    let bytes = col.hit_bytes_mut(hit);
                                    for b in bytes.iter_mut() {
                                        *b = reverse_complement_iupac(&[*b])[0];
                                    }
                                }
                            }
                        }

                        TagColumn::String(bstrings) => {
                            for bstring in bstrings.iter_mut() {
                                if let Some(s) = bstring.as_mut() {
                                    *s = reverse_complement_iupac(s).into();
                                }
                            }
                        }

                        TagColumn::Numeric(_) | TagColumn::Bool(_) => unreachable!(), // cov:excl-line
                    }
                } // cov:excl-line    
            }
            ResolvedSourceAll::Name { .. } => unreachable!(), // cov:excl-line
        }

        Ok((block, true))
    }
}
