use crate::transformations::prelude::*;

use crate::{dna::TagValue, io};

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
        self.out_label.verify(|v| {
            if v.0.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
        self.in_label.verify(|v| {
            if v.0.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
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
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");
        TagUsageInfo {
            used_tags: vec![inner.in_label.to_used_tag(&[TagValueType::Location])],
            declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric),
            ..Default::default()
        }
    }
}

impl Step for RegionsToLength {
    fn apply(
        &self,
        mut block: io::FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        let region_values = block.tags.get(&self.in_label).cloned().ok_or_else(|| {
            anyhow!(
                "ConvertRegionsToLength expects region tag '{}' to be available",
                self.in_label
            )
        })?;

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
                other => {
                    bail!(
                        "ConvertRegionsToLength expected '{}' to contain region tags, found {:?}",
                        self.in_label,
                        other
                    );
                }
            };
            #[allow(clippy::cast_precision_loss)]
            lengths.push(TagValue::Numeric(length as f64));
        }

        block.tags.insert(self.out_label.clone(), lengths);
        Ok((block, true))
    }
}
