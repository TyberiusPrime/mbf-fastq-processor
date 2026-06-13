use bstr::ByteSlice;

use crate::transformations::prelude::*;
use fastqrab_config::{default_region_separator, tpd_adapt_bstring};

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
    #[schemars(with = "String")]
    #[tpd(with = "tpd_adapt_bstring")]
    separator: BString,

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
        self.separator.or_with(default_region_separator);
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
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut()
            && let Some(tv_in_labels) = inner.in_labels.value.as_mut()
        {
            let used_tags: Vec<_> = tv_in_labels
                .iter_mut()
                .filter(|x| x.is_ok())
                .map(|x| x.to_used_tag(&[TagValueType::Location, TagValueType::String]))
                .collect();

            Some(TagUsageInfo {
                used_tags,
                declared_tag: inner.out_label.to_declared_tag(TagValueType::String),
                must_see_all_tags: true,
                ..Default::default()
            })
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
        let num_reads = block.segments[0].len();

        let output_tags = {
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

            let mut string_iters: Vec<_> = tag_columns
                .into_iter()
                .map(|x| x.iter_bstr(|number| number.to_string(), Some(self.separator.as_bytes())))
                .collect();

            // Output is String (convert Location to sequence bytes)
            let mut output_tags: Vec<Option<BString>> = Vec::with_capacity(num_reads);
            match self.on_missing {
                OnMissing::MergePresent => {
                    for _read_idx in 0..num_reads {
                        let parts: Vec<_> = string_iters
                            .iter_mut()
                            .map(|iter| iter.next().expect("tag length != block length!?"))
                            .collect();
                        let mut new_value: BString = BString::new(Vec::with_capacity(
                            parts.iter().map(|x| x.len()).sum::<usize>()
                                + (self.separator.len() * (parts.len() - 1)),
                        )); // initial
                        let mut first = true;
                        for p in parts {
                            if !first && !self.separator.is_empty() {
                                new_value.extend_from_slice(self.separator.as_bytes());
                            } else {
                                first = false;
                            }
                            new_value.extend_from_slice(&p);
                        }
                        output_tags.push(Some(new_value));
                    }
                }
                OnMissing::SetMissing => {
                    for _read_idx in 0..num_reads {
                        let parts: Vec<_> = string_iters
                            .iter_mut()
                            .map(|iter| iter.next().expect("tag length != block length!?"))
                            .collect();
                        if parts.iter().any(|x| x.is_empty()) {
                            output_tags.push(None);
                        } else {
                            let mut new_value: BString = BString::new(Vec::with_capacity(
                                parts.iter().map(|x| x.len()).sum::<usize>()
                                    + self.separator.len() * (parts.len() - 1),
                            )); // initial
                            let mut first = true;
                            for p in parts {
                                if !first && !self.separator.is_empty() {
                                    new_value.extend_from_slice(&self.separator.as_bytes());
                                } else {
                                    first = false;
                                }
                                new_value.extend_from_slice(&p);
                            }
                            output_tags.push(Some(new_value));
                        }
                    }
                }
            }
            output_tags
        };
        block
            .tags
            .insert(self.out_label.clone(), TagColumn::String(output_tags));

        Ok((block, true))
    }
}
