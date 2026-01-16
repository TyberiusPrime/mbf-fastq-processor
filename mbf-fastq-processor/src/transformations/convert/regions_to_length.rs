use crate::transformations::prelude::*;

use crate::{dna::TagValue, io};

/// Convert region tag to a length tag
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegionsToLength {
    pub out_label: String,
    pub in_label: String,
}

impl Step for RegionsToLength {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.out_label == self.in_label {
            bail!(
                "ConvertRegionsToLength: 'label' must differ from 'region_label' to avoid overwriting the source tag"
            );
        }
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.out_label.clone(), TagValueType::Numeric))
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
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
