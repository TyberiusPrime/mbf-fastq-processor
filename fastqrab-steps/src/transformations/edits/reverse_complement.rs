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
        if matches!(
            self.target.as_ref(),
            Some(MustAdapt::PostVerify(ResolvedSourceAll::Name { .. }))
        ) {
            self.target.state =
                TomlValueState::new_validation_failed("Must not be 'name:' definition");
            self.target.help =
                Some("ReverseComplement does not support name-based targeting".to_string());
        }
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
    #[allow(clippy::redundant_closure_for_method_calls)] // otherwise the FnOnce is not general
    // enough
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
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
                    |location: &HitRegion, _pos, seq: &BString, read_len: usize| -> NewLocation {
                        let new_start = read_len.checked_sub(location.start + location.len).expect("Start position underflow");
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
                            block.filter_tag_locations(SegmentIndex(idx), ftl, condition.as_deref())
                        }
                    }
                    SegmentIndexOrAll::Indexed(segment) => block.filter_tag_locations(
                        SegmentIndex(*segment),
                        ftl,
                        condition.as_deref(),
                    ),
                }
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    for tag_val in hits.iter_mut() {
                        match tag_val {
                            TagValue::Missing => {}
                            TagValue::Location(hits) => {
                                for hit_region in &mut hits.0 {
                                    for ii in 0..hit_region.sequence.len() {
                                        hit_region.sequence[ii] =
                                            reverse_complement_iupac(&[hit_region.sequence[ii]])[0]
                                    }
                                }
                            }
                            TagValue::String(bstring) => {
                                *bstring = reverse_complement_iupac(bstring).into();
                            }
                            TagValue::Numeric(_) | TagValue::Bool(_) => unreachable!(),
                        }
                    }
                } // cov:excl-line    
            }
            ResolvedSourceAll::Name { .. } => unreachable!(),
        }

        Ok((block, true))
    }
}
