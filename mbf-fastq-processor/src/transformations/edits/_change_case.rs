#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;

use crate::dna::TagValue;

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

    pub if_tag: Option<ConditionalTagLabel>,
}

impl Partial_ChangeCase {
    pub fn new(
        target: MustAdapt<String, ResolvedSourceAll>,
        case_type: CaseType,
        if_tag: Option<ConditionalTagLabel>,
    ) -> Self {
        Self {
            target: TomlValue::new_ok_unplaced(target),
            case_type: TomlValue::new_ok_unplaced(case_type),
            if_tag: TomlValue::new_ok_unplaced(if_tag),
        }
    }
}

impl VerifyIn<PartialConfig> for Partial_ChangeCase {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.target.validate_segment(parent);
        Ok(())
    }
}

impl _ChangeCase {
    pub fn new(
        target: ResolvedSourceAll,
        case_type: CaseType,
        if_tag: Option<ConditionalTagLabel>,
    ) -> Self {
        Self {
            target,
            case_type,
            if_tag,
        }
    }
}

impl TagUser for PartialTaggedVariant<Partial_ChangeCase> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        let mut used_tags = vec![inner.if_tag.to_used_tag(&[][..])];
        used_tags.extend(inner.target.to_used_tags());

        TagUsageInfo {
            used_tags,
            ..Default::default()
        }
    }
}

impl Step for _ChangeCase {
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
                        let seq = read.seq().to_vec();
                        let new_seq: Vec<u8> = seq.iter().map(|&b| case_converter(b)).collect();
                        read.replace_seq_keep_qual(&new_seq);
                    },
                    condition.as_deref(),
                );
            }
            ResolvedSourceAll::Tag(tag_name) => {
                if let Some(hits) = block.tags.get_mut(tag_name) {
                    for tag_val in hits.iter_mut() {
                        if let TagValue::Location(hit) = tag_val {
                            for hit_region in &mut hit.0 {
                                for ii in 0..hit_region.sequence.len() {
                                    hit_region.sequence[ii] =
                                        case_converter(hit_region.sequence[ii]);
                                }
                            }
                        }
                    }
                }
            }
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                block.apply_in_place_wrapped_plus_all(
                    *segment_index_or_all,
                    |read| {
                        let new: Vec<u8> = read
                            .name_without_comment(*split_character)
                            .to_vec()
                            .into_iter()
                            .map(case_converter)
                            .collect();
                        if let Some(comment) = read.name_only_comment(*split_character) {
                            let mut full = new;
                            full.push(*split_character);
                            full.extend(comment);
                            read.replace_name(&full);
                        } else {
                            read.replace_name(&new);
                        }
                    },
                    condition.as_deref(),
                );
            }
        }

        Ok((block, true))
    }
}
