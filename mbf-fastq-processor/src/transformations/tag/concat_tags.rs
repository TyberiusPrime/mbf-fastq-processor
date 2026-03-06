#![allow(clippy::unnecessary_wraps)]

//eserde false positives
use crate::transformations::prelude::*;

use crate::dna::{Hit, Hits, TagValue};

/// Behavior when encountering missing tags during concatenation
#[derive(Clone, JsonSchema, PartialEq, Eq, Copy)]
#[tpd]
#[derive(Debug)]
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

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ConcatTags {
    /// Input tag labels to concatenate (must have at least 2)
    in_labels: Vec<TagLabel>,

    /// Output tag label for the concatenated result
    out_label: TagLabel,

    /// Separator to use when concatenating strings (optional, defaults to empty)
    separator: Option<String>,

    /// Behavior when encountering missing tags
    /// - `merge_present`: Skip missing tags and merge only the present ones
    /// - `set_missing`: Set the output tag to missing if any input tag is missing
    on_missing: OnMissing,
}

impl VerifyIn<PartialConfig> for PartialConcatTags {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.in_labels.verify_mut(|v| {
            // if v.len() < 2 {
            // don't check here, check  in get_tag_usage, so we can make suggestions
            // }
            let mut seen = std::collections::HashSet::new();
            for label in v.iter_mut() {
                let lv = label.value.as_ref().expect("Parent was ok?");
                if lv.is_empty() {
                    label.state = TomlValueState::ValidationFailed {
                        message: "Must not be empty".to_string(),
                    };
                } else if !seen.insert(lv) {
                    label.state = TomlValueState::ValidationFailed {
                        message: "Duplicate input label".to_string(),
                    };
                }
            }
            Ok(())
        });
        self.out_label.verify(|v| {
            if v.is_empty() {
                Err(ValidationFailure::new(
                    "Output label must not be empty",
                    None,
                ))
            } else {
                Ok(())
            }
        });

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialConcatTags> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> TagUsageInfo<'_> {
        let inner = self
            .toml_value
            .as_mut()
            .expect("get_tag_usage should only be called after successful verification");

        let in_labels: Vec<TagLabel> = {
            let tv_in_labels = inner.in_labels.as_ref().expect("Parent was ok?");
            tv_in_labels
                .iter()
                .filter_map(|v| v.value.as_ref())
                .cloned()
                .collect()
        };
        if in_labels.len() < 2 {
            //we do this here so we can make suggestions.
            let mut available: Vec<String> = tags_available
                .iter()
                .filter_map(|(tag_name, tag_meta)| {
                    if !in_labels.contains(tag_name)
                        && matches!(
                            tag_meta.tag_type,
                            TagValueType::Location | TagValueType::String
                        )
                    {
                        Some(tag_name.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            available.sort_unstable();

            inner.in_labels.state = TomlValueState::ValidationFailed {
                message: "Must have at least two input labels".to_string(),
            };
            inner.in_labels.help = Some(format!(
                "Provide at least two tags to concatenate. Available: {}",
                available.join(", ")
            ));
            TagUsageInfo::default()
        } else {
            let tv_in_labels = inner.in_labels.as_mut().expect("Parent was ok?");
            let all_location = tv_in_labels.iter().all(|v| {
                if let Some(lv) = v.value.as_ref() {
                    matches!(
                        tags_available.get(lv).map(|meta| &meta.tag_type),
                        Some(TagValueType::Location)
                    )
                } else {
                    true // skip invalid labels, they will be caught in validation
                }
            });
            let output_type = if all_location {
                TagValueType::Location
            } else {
                TagValueType::String
            };
            let used_tags: Vec<_> = tv_in_labels
                .iter_mut()
                .map(|x| x.to_used_tag(&[TagValueType::Location, TagValueType::String]))
                .collect();

            TagUsageInfo {
                used_tags,
                declared_tag: inner.out_label.to_declared_tag(output_type),
                must_see_all_tags: true,
                ..Default::default()
            }
        }
    }
}

impl Step for ConcatTags {
    #[allow(clippy::too_many_lines)]
    fn apply(
        &self,
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
                    .ok_or_else(|| anyhow::anyhow!("Tag '{label}' not found in block"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Process each read
        for read_idx in 0..num_reads {
            // Collect tag values for this read

            let mut any_missing = false;
            let mut has_location = false;
            let mut has_string = false;

            let tag_values = tag_vectors.iter().map(|vec| &vec[read_idx]);

            for tv in tag_values {
                match tv {
                    TagValue::Missing => {
                        any_missing = true;
                    }
                    TagValue::Location(_) => {
                        has_location = true;
                    }
                    TagValue::String(_) => {
                        has_string = true;
                    }
                    TagValue::Numeric(_) | TagValue::Bool(_) => {
                        bail!(
                            "ConcatTags does not support Numeric or Bool tags. Found in one of: {:?}",
                            self.in_labels
                        );
                    }
                }
            }
            // Handle missing tags according to on_missing setting
            if any_missing {
                match self.on_missing {
                    OnMissing::SetMissing => {
                        // If any tag is missing, set output to missing
                        output_tags.push(TagValue::Missing);
                        continue;
                    }
                    OnMissing::MergePresent => {
                        // continue to merge present tags - if none, the lower code will handle it
                    }
                }
            }
            // Case 1: All tags are Location (or some are Missing)
            // Concatenate regions and sequences
            if has_location && !has_string {
                let mut combined_hits: Vec<Hit> = Vec::with_capacity(tag_vectors.len());

                let tag_values = tag_vectors.iter().map(|vec| &vec[read_idx]);
                for tag_value in tag_values {
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
                let mut parts: Vec<&[u8]> = Vec::with_capacity(tag_vectors.len());

                let tag_values = tag_vectors.iter().map(|vec| &vec[read_idx]);
                for tag_value in tag_values {
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
                let mut parts: Vec<Vec<u8>> = Vec::with_capacity(tag_vectors.len());

                let tag_values = tag_vectors.iter().map(|vec| &vec[read_idx]);
                for tag_value in tag_values {
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
                    let parts_refs: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
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
