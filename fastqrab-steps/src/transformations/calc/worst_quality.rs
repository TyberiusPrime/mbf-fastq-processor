use super::extract_numeric_tags_plus_all;
use crate::transformations::prelude::*;
use fastqrab_io::io::WrappedFastQRead;

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
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Numeric((None, None))),
                used_tags: inner.source.to_used_tags(),
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
                extract_numeric_tags_plus_all(
                    *seg_or_all,
                    &self.out_label,
                    |read| min_quality(read, self.offset) as f64,
                    |reads| {
                        reads
                            .iter()
                            .map(|r| min_quality(r, self.offset))
                            .min()
                            .unwrap_or(33) as f64
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

                let location_items = match &tag_values {
                    TagColumn::Location(items) => items,
                    _ => anyhow::bail!("WorstQuality source tag must be a Location column"),
                };
                let missing_value = 33.0 + self.offset as f64;
                let mut values = Vec::with_capacity(location_items.len());
                let mut iter = block.get_pseudo_iter();

                for opt_hits in location_items {
                    let molecule = iter.pseudo_next().expect("tag and read count should match");
                    let q = match opt_hits {
                        Some(hits) => match molecule.hit_to_qualities(hits) {
                            Some(qual_bytes) if !qual_bytes.is_empty() => qual_bytes
                                .iter()
                                .map(|x| Into::<i16>::into(*x) + self.offset as i16)
                                .min()
                                .unwrap_or(33 + self.offset as i16)
                                as f64,
                            _ => missing_value,
                        },
                        None => missing_value,
                    };
                    values.push(q);
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

fn min_quality(read: &WrappedFastQRead, offset: i8) -> i16 {
    read.qual()
        .iter()
        .map(|x| Into::<i16>::into(*x) + offset as i16)
        .min()
        .unwrap_or(33)
}
