use super::extract_numeric_tags_plus_all_from_qualities;
use crate::transformations::prelude::*;

/// Calculate minimum quality byte value across a segment or a tagged region
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct WorstQuality {
    pub out_label: TagLabel,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    #[tpd(alias = "segment")]
    pub source: ResolvedSourceAll,

    #[tpd(default)]
    pub offset: i8,
}

impl VerifyIn<PartialConfig> for PartialWorstQuality {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        self.source
            .deny_name("CalcWorstQuality does not support name-based targeting");
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialWorstQuality> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = inner.source.to_used_tags();
            for ut in used_tags.iter_mut().flatten() {
                ut.accepted_tag_types = &[TagValueType::Location];
            }
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Numeric((None, None))),
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for WorstQuality {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        match &self.source {
            ResolvedSourceAll::Segment(seg_or_all) => {
                extract_numeric_tags_plus_all_from_qualities(
                    *seg_or_all,
                    &self.out_label,
                    |read| f64::from(min_quality(read, self.offset)),
                    |reads| {
                        f64::from(
                            reads
                                .iter()
                                .map(|r| min_quality(r, self.offset))
                                .min()
                                .expect("No segments? That's unexpected"),
                        )
                    },
                    &mut block,
                );
            }
            ResolvedSourceAll::Tag(label) => {
                let tag_values = block
                    .tags
                    .get(label)
                    .expect("source tag not found — should have been caught in validation")
                    .clone();

                let TagColumn::Location(location_items) = &tag_values else {
                    unreachable!(
                        "WorstQuality source tag must be a Location column, and verify should check that"
                    );
                };

                let mut values = Vec::with_capacity(location_items.row_count());
                {
                    for qual in location_items.iter_qual() {
                        let q = f64::from(
                            qual.iter()
                                .map(|x| Into::<i16>::into(*x) + i16::from(self.offset))
                                .min()
                                .unwrap_or(33 + i16::from(self.offset)),
                        );
                        values.push(q);
                    }
                }

                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::Numeric(values));
            }
            ResolvedSourceAll::Name { .. } => unreachable!(), // cov:excl-line
        }

        Ok((block, true))
    }
}

fn min_quality(quality: &BStr, offset: i8) -> i16 {
    quality
        .iter()
        .map(|x| Into::<i16>::into(*x))
        .min()
        .unwrap_or(33)
        + Into::<i16>::into(offset)
}
