use anyhow::{Result, anyhow, bail};

use crate::{demultiplex::Demultiplexed, dna::TagValue, io};

use super::super::{Step, TagValueType, Transformation};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConvertRegionsToLength {
    pub label: String,
    pub region_label: String,
}

impl Step for ConvertRegionsToLength {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.label == self.region_label {
            bail!(
                "ConvertRegionsToLength: 'label' must differ from 'region_label' to avoid overwriting the source tag"
            );
        }
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.label.clone(), TagValueType::Numeric))
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.region_label.clone(), &[TagValueType::Location])])
    }

    fn apply(
        &mut self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        let tags = block.tags.as_mut().ok_or_else(|| {
            anyhow!(
                "ConvertRegionsToLength expects region tag '{}' to be available",
                self.region_label
            )
        })?;

        let region_values = tags.get(&self.region_label).cloned().ok_or_else(|| {
            anyhow!(
                "ConvertRegionsToLength expects region tag '{}' to be available",
                self.region_label
            )
        })?;

        let mut lengths: Vec<TagValue> = Vec::with_capacity(region_values.len());
        for tag_value in region_values {
            let length = match tag_value {
                TagValue::Sequence(hits) => hits
                    .0
                    .iter()
                    .map(|hit| {
                        hit.location
                            .as_ref()
                            .map(|loc| loc.len)
                            .unwrap_or_else(|| hit.sequence.len())
                    })
                    .sum::<usize>(),
                TagValue::Missing => 0,
                other => {
                    bail!(
                        "ConvertRegionsToLength expected '{}' to contain region tags, found {:?}",
                        self.region_label,
                        other
                    );
                }
            };
            #[allow(clippy::cast_precision_loss)]
            lengths.push(TagValue::Numeric(length as f64));
        }

        tags.insert(self.label.clone(), lengths);
        Ok((block, true))
    }
}
