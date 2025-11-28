#![allow(clippy::unnecessary_wraps)]
use std::cell::OnceCell;

//eserde false positives
use crate::transformations::prelude::*;

use crate::dna::{Hit, Hits, TagValue};

/// Behavior when encountering missing tags during concatenation
#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OnMissing {
    /// Skip missing tags and merge only the present ones
    MergePresent,
    /// Set the output tag to missing if any input tag is missing
    SetMissing,
}

/// Concatenate multiple tags into a single tag.
///
/// Takes n >= 2 tags (which can be location tags or string tags) and combines them:
/// - If all tags are Location: appends regions and concatenates sequences
/// - If all tags are String: concatenates strings with optional separator
/// - If mixed (Location + String): converts all to strings and concatenates
///
/// # Examples
///
/// ```toml
/// [[step]]
/// action = "ConcatTags"
/// in_labels = ["barcode1", "barcode2"]
/// out_label = "combined_barcode"
/// separator = "_"  # Optional, only used for string concatenation
/// ```
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConcatTags {
    /// Input tag labels to concatenate (must have at least 2)
    #[validate(min_items = 2)]
    in_labels: Vec<String>,

    /// Output tag label for the concatenated result
    out_label: String,

    #[schemars(skip)]
    #[serde(default)]
    #[serde(skip)]
    output_tag_type: OnceCell<TagValueType>,

    /// Separator to use when concatenating strings (optional, defaults to empty)
    #[serde(default)]
    separator: Option<String>,

    /// Behavior when encountering missing tags
    /// - merge_present: Skip missing tags and merge only the present ones
    /// - set_missing: Set the output tag to missing if any input tag is missing
    on_missing: OnMissing,
}

impl Step for ConcatTags {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        if self.in_labels.len() < 2 {
            bail!(
                "ConcatTags requires at least 2 input tags, got {}",
                self.in_labels.len()
            );
        }

        // Check for duplicate input labels
        let mut seen = std::collections::HashSet::new();
        for label in &self.in_labels {
            if !seen.insert(label) {
                bail!("ConcatTags has duplicate input label: {}", label);
            }
        }

        // Validate that all input tags exist before this step
        let mut all_location = true;
        for label in &self.in_labels {
            let mut found = false;
            for (idx, transform) in all_transforms.iter().enumerate() {
                if idx >= this_transforms_index {
                    break;
                }
                if let Some((tag_name, tag_type)) = transform.declares_tag_type() {
                    if tag_name == *label {
                        found = true;
                        match tag_type {
                            TagValueType::Location => {}
                            TagValueType::String => {
                                all_location = false;
                            }
                            _ => {
                                continue; // check for invalid type is done in uses_tags, don't want
                                // separate error messages
                            }
                        }
                        break;
                    }
                }
            }
            if !found {
                bail!(
                    "ConcatTags requires tag '{}' but it is not declared before this step",
                    label
                );
            }
        }
        if all_location {
            self.output_tag_type
                .set(TagValueType::Location)
                .expect("Trying to set output_tag_type twice")
        } else {
            self.output_tag_type
                .set(TagValueType::String)
                .expect("Trying to set output_tag_type twice");
        }

        Ok(())
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(
            self.in_labels
                .iter()
                .map(|label| {
                    (
                        label.clone(),
                        &[TagValueType::Location, TagValueType::String][..],
                    )
                })
                .collect(),
        )
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        // We'll determine the output type dynamically based on input types
        // The actual type will be set during init based on input tags at runtime
        Some((
            self.out_label.clone(),
            self.output_tag_type
                .get()
                .map(|x| *x)
                .expect("output_tag_type should be set during validation"),
        ))
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let num_reads = block.segments[0].entries.len();
        let mut output_tags = Vec::with_capacity(num_reads);

        // Collect tag vectors for all input labels
        let tag_vectors: Vec<&Vec<TagValue>> = self
            .in_labels
            .iter()
            .map(|label| {
                block
                    .tags
                    .get(label)
                    .ok_or_else(|| anyhow::anyhow!("Tag '{}' not found in block", label))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Process each read
        for read_idx in 0..num_reads {
            // Collect tag values for this read
            let tag_values: Vec<&TagValue> = tag_vectors.iter().map(|vec| &vec[read_idx]).collect();

            // Check if any tags are missing
            let any_missing = tag_values.iter().any(|tv| tv.is_missing());

            // Handle missing tags according to on_missing setting
            if any_missing {
                match self.on_missing {
                    OnMissing::SetMissing => {
                        // If any tag is missing, set output to missing
                        output_tags.push(TagValue::Missing);
                        continue;
                    }
                    OnMissing::MergePresent => {
                        // Check if all tags are missing
                        let all_missing = tag_values.iter().all(|tv| tv.is_missing());
                        if all_missing {
                            output_tags.push(TagValue::Missing);
                            continue;
                        }
                        // Otherwise, continue to merge present tags
                    }
                }
            }

            // Determine the types of non-missing tags
            let has_location = tag_values
                .iter()
                .any(|tv| matches!(tv, TagValue::Location(_)));
            let has_string = tag_values
                .iter()
                .any(|tv| matches!(tv, TagValue::String(_)));
            let has_other = tag_values
                .iter()
                .any(|tv| matches!(tv, TagValue::Numeric(_) | TagValue::Bool(_)));

            if has_other {
                bail!(
                    "ConcatTags does not support Numeric or Bool tags. Found in one of: {:?}",
                    self.in_labels
                );
            }

            // Case 1: All tags are Location (or some are Missing)
            // Concatenate regions and sequences
            if has_location && !has_string {
                let mut combined_hits: Vec<Hit> = Vec::new();

                for tag_value in &tag_values {
                    match tag_value {
                        TagValue::Location(hits) => {
                            combined_hits.extend(hits.0.iter().cloned());
                        }
                        TagValue::Missing => {
                            // Skip missing tags
                        }
                        _ => unreachable!("Should only have Location or Missing"),
                    }
                }

                if combined_hits.is_empty() {
                    output_tags.push(TagValue::Missing);
                } else {
                    output_tags.push(TagValue::Location(Hits::new_multiple(combined_hits)));
                }
            }
            // Case 2: All tags are String (or some are Missing)
            // Concatenate strings with separator
            else if has_string && !has_location {
                let mut parts: Vec<&[u8]> = Vec::new();

                for tag_value in &tag_values {
                    match tag_value {
                        TagValue::String(s) => {
                            parts.push(s.as_ref());
                        }
                        TagValue::Missing => {
                            // Skip missing tags
                        }
                        _ => unreachable!("Should only have String or Missing"),
                    }
                }

                if parts.is_empty() {
                    output_tags.push(TagValue::Missing);
                } else {
                    let result = if let Some(sep) = &self.separator {
                        parts.join(sep.as_bytes())
                    } else {
                        parts.concat()
                    };
                    output_tags.push(TagValue::String(result.into()));
                }
            }
            // Case 3: Mixed Location and String
            // Convert all to strings and concatenate
            else {
                let mut parts: Vec<Vec<u8>> = Vec::new();

                for tag_value in &tag_values {
                    match tag_value {
                        TagValue::Location(hits) => {
                            // Convert location to sequence string
                            parts.push(hits.joined_sequence(None));
                        }
                        TagValue::String(s) => {
                            parts.push(s.to_vec());
                        }
                        TagValue::Missing => {
                            // Skip missing tags
                        }
                        _ => unreachable!("Should only have Location, String, or Missing"),
                    }
                }

                if parts.is_empty() {
                    output_tags.push(TagValue::Missing);
                } else {
                    let parts_refs: Vec<&[u8]> = parts.iter().map(|v| v.as_slice()).collect();
                    let result = if let Some(sep) = &self.separator {
                        parts_refs.join(sep.as_bytes())
                    } else {
                        parts_refs.concat()
                    };
                    output_tags.push(TagValue::String(result.into()));
                }
            }
        }

        // Insert the concatenated tags into the block
        block.tags.insert(self.out_label.clone(), output_tags);

        Ok((block, true))
    }
}
