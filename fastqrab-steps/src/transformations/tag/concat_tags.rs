use crate::transformations::prelude::*;

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
            let mut seen: IndexMap<&TagLabel, std::ops::Range<usize>> = IndexMap::new();
            for label in v.iter_mut() {
                let lv = label.value.as_ref().expect("Parent was ok?");
                match seen.entry(lv) {
                    indexmap::map::Entry::Occupied(occupied_entry) => {
                        let spans = vec![
                            (label.span(), "Duplicate input label".to_string()),
                            (occupied_entry.get().clone(), "First occurrence".to_string()),
                        ];
                        label.state = TomlValueState::Custom { spans };
                    }
                    indexmap::map::Entry::Vacant(vacant_entry) => {
                        vacant_entry.insert(label.span());
                    }
                }
            }
            Ok(())
        });

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialConcatTags> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut()
            && let Some(tv_in_labels) = inner.in_labels.value.as_mut()
        {
            let in_labels: Vec<TagLabel> = {
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
                None
            } else {
                let all_location = in_labels.iter().all(|v| {
                    matches!(
                        tags_available.get(v).map(|meta| &meta.tag_type),
                        Some(TagValueType::Location)
                    )
                });
                let output_type = if all_location {
                    TagValueType::Location
                } else {
                    TagValueType::String
                };
                let used_tags: Vec<_> = tv_in_labels
                    .iter_mut()
                    .filter(|x| x.is_ok())
                    .map(|x| x.to_used_tag(&[TagValueType::Location, TagValueType::String]))
                    .collect();

                Some(TagUsageInfo {
                    used_tags,
                    declared_tag: inner.out_label.to_declared_tag(output_type),
                    must_see_all_tags: true,
                    ..Default::default()
                })
            }
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ConcatTags {
    #[expect(clippy::too_many_lines, reason = "it takes that many")]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let num_reads = block.segments[0].entries.len();

        // Collect tag columns for all input labels
        let tag_columns: Vec<&TagColumn> = self
            .in_labels
            .iter()
            .map(|label| {
                block
                    .tags
                    .get(label)
                    .ok_or_else(|| anyhow::anyhow!("Tag '{label}' not found in block"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        // Determine if all columns are Location type
        let all_location = tag_columns
            .iter()
            .all(|col| matches!(col, TagColumn::Location(_)));

        if all_location {
            // Output is Location
            let mut output_tags: Vec<Option<Hits>> = Vec::with_capacity(num_reads);
            for read_idx in 0..num_reads {
                let mut any_missing = false;
                let mut combined_hits: Vec<Hit> = Vec::new();
                for col in &tag_columns {
                    let item = col.get_location(read_idx);
                    match item {
                        None => any_missing = true,
                        Some(hits) => combined_hits.extend(hits.0.iter().cloned()),
                    }
                }
                if any_missing && self.on_missing == OnMissing::SetMissing {
                    output_tags.push(None);
                } else if combined_hits.is_empty() {
                    output_tags.push(None);
                } else {
                    output_tags.push(Some(Hits::new_multiple(combined_hits)));
                }
            }
            block
                .tags
                .insert(self.out_label.clone(), TagColumn::Location(output_tags));
        } else {
            // Output is String (convert Location to sequence bytes)
            let mut output_tags: Vec<Option<BString>> = Vec::with_capacity(num_reads);
            for read_idx in 0..num_reads {
                let mut any_missing = false;
                let mut parts: Vec<Vec<u8>> = Vec::new();
                for col in &tag_columns {
                    match col {
                        TagColumn::Location(items) => match &items[read_idx] {
                            None => any_missing = true,
                            Some(hits) => parts.push(hits.joined_sequence(None)),
                        },
                        TagColumn::String(items) => match &items[read_idx] {
                            None => any_missing = true,
                            Some(s) => parts.push(s.to_vec()),
                        },
                        // cov:excl-start
                        TagColumn::Numeric(_) | TagColumn::Bool(_) => {
                            bail!(
                                "ConcatTags does not support Numeric or Bool tags. Found in one of: {:?}",
                                self.in_labels
                            );
                        } // cov:excl-stop
                    }
                }
                if any_missing && self.on_missing == OnMissing::SetMissing {
                    output_tags.push(None);
                } else if parts.is_empty() {
                    output_tags.push(None);
                } else {
                    let parts_refs: Vec<&[u8]> = parts.iter().map(Vec::as_slice).collect();
                    let result = if let Some(sep) = &self.separator {
                        parts_refs.join(sep.as_bytes())
                    } else {
                        parts_refs.concat()
                    };
                    output_tags.push(Some(result.into()));
                }
            }
            block
                .tags
                .insert(self.out_label.clone(), TagColumn::String(output_tags));
        }

        Ok((block, true))
    }
}
