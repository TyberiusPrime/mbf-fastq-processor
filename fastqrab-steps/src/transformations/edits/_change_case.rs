use fastqrab_io::blocks::split_name_and_comment_mut;

use crate::transformations::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[tpd]
#[derive(Default)]
pub enum CaseType {
    #[default]
    Lower,
    Upper,
}

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct _ChangeCase {
    #[tpd(alias = "segment")]
    #[tpd(alias = "source")]
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    target: ResolvedSourceAll,

    #[tpd(default)]
    #[schemars(skip)]
    case_type: CaseType,

    #[tpd(alias = "if_label")]
    pub if_tag: Option<ConditionalTagLabel>,
}

impl Partial_ChangeCase {
    pub fn new(
        target: TomlValue<MustAdapt<String, ResolvedSourceAll>>,
        case_type: CaseType,
        if_tag: TomlValue<Option<ConditionalTagLabel>>,
    ) -> Self {
        Self {
            target,
            case_type: TomlValue::new_ok_unplaced(case_type),
            if_tag,
        }
    }
}

impl VerifyIn<PartialConfig> for Partial_ChangeCase {
    // cov:excl-start
    #[mutants::skip]
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        unreachable!(); //since this is in expanded step.
        // self.target.validate_segment(parent);
        // Ok(())
    }
    // cov:excl-stop
}

impl TagUser for PartialTaggedVariant<Partial_ChangeCase> {
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
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for _ChangeCase {
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

        let resolved_source = &self.target;

        let case_converter: fn(u8) -> u8 = match self.case_type {
            CaseType::Lower => |b| b.to_ascii_lowercase(),
            CaseType::Upper => |b| b.to_ascii_uppercase(),
        };

        match resolved_source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        for b in read.seq.iter_mut() {
                            *b = case_converter(*b);
                        }
                    },
                    condition.as_deref(),
                );
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    match hits {
                        // Changing a tag's case changes the *tag's* contents, not
                        // the read. The row COW-detaches into owned bytes; the read
                        // only changes via an explicit write-back step.
                        TagColumn::Location(col) => {
                            for idx in 0..col.row_count() {
                                if col.row_is_empty(idx) {
                                    continue;
                                }
                                if condition.as_ref().is_some_and(|c| !c[idx]) {
                                    continue;
                                }
                                let mut new_seq = col.joined_seq(idx, None).to_vec();
                                for b in &mut new_seq {
                                    *b = case_converter(*b);
                                }
                                let new_qual = col.joined_qual(idx, None).to_vec();
                                // The row keeps its existing anchor (the original
                                // references); only the bytes diverge.
                                col.set_row_content(idx, &new_seq, &new_qual);
                            }
                        }
                        TagColumn::String(opt_bstrings) => {
                            for (idx, slot) in opt_bstrings.iter_mut().enumerate() {
                                if condition.as_ref().is_some_and(|c| !c[idx]) {
                                    continue;
                                }
                                if let Some(value) = slot {
                                    for b in value.iter_mut() {
                                        *b = case_converter(*b);
                                    }
                                }
                            }
                        }
                        TagColumn::Numeric(_) | TagColumn::Bool(_) => panic!(
                            "Can't convert case on non-string tags. Should have been caught in validation"
                        ),
                    }
                } // cov:excl-line
            }
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        let (name, _comment) =
                            split_name_and_comment_mut(read.name, *split_character);
                        for b in name.iter_mut() {
                            *b = case_converter(*b);
                        }
                    },
                    condition.as_deref(),
                );
            }
        }

        Ok((block, true))
    }
}
