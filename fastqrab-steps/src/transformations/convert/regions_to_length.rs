use crate::transformations::prelude::*;
use fastqrab_dna::dna::TagValue;

/// Convert region tag to a length tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct RegionsToLength {
    pub out_label: TagLabel,
    pub in_label: TagLabel,
}

impl VerifyIn<PartialConfig> for PartialRegionsToLength {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(out) = self.out_label.as_ref()
            && let Some(in_) = self.in_label.as_ref()
            && out == in_
        {
            let spans = vec![
                (self.out_label.span(), "Same as in_label".to_string()),
                (self.in_label.span(), "Same as out_label".to_string()),
            ];
            self.out_label.state = TomlValueState::Custom { spans };
            self.out_label.help = Some(
                "'out_label' must differ from 'in_label' to avoid overwriting the source tag."
                    .to_string(),
            );
        }
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialRegionsToLength> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location])],
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric((
                    Some(NonNaN::new(0.0).expect("can't fail")),
                    None,
                ))),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for RegionsToLength {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let region_values = block.tags.get(&self.in_label).cloned().expect(
                "ConvertRegionsToLength expects region tag to be available -should have been caught in validation. Bug",
            );

        let mut lengths: Vec<TagValue> = Vec::with_capacity(region_values.len());
        for tag_value in region_values {
            let length = match tag_value {
                TagValue::Location(hits) => hits
                    .0
                    .iter()
                    .map(|hit| {
                        hit.location
                            .as_ref()
                            .map_or_else(|| hit.sequence.len(), |loc| loc.len)
                    })
                    .sum::<usize>(),
                TagValue::Missing => 0,
                // cov:excl-start
                other => {
                    panic!(
                        "ConvertRegionsToLength expected '{}' to contain region tags, found {:?}. Should have been caught in validation",
                        self.in_label, other
                    );
                } // cov:excl-stop
            };
            #[expect(clippy::cast_precision_loss, reason = "it's going to be small enough")]
            lengths.push(TagValue::Numeric(length as f64));
        }

        block.tags.insert(self.out_label.clone(), lengths);
        Ok((block, true))
    }
}
