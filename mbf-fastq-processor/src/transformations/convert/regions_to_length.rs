use crate::transformations::prelude::*;

use crate::{dna::TagValue, io};

/// Convert region tag to a length tag

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct RegionsToLength {
    pub out_label: String,
    pub in_label: String,
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
            if v.is_empty() {
                Err(ValidationFailure::new("Must not be empty", None))
            } else {
                Ok(())
            }
        });
        self.in_label.verify(|v| {
            if v.is_empty() {
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

impl Step for RegionsToLength {
    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.out_label.clone(), TagValueType::Numeric))
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

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
