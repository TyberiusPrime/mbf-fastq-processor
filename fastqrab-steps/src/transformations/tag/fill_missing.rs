use crate::transformations::prelude::*;

/// Return the primary tag if it is not missing, otherwise return the secondary tag.
///
/// Both tags must have the same kind (Location or String), or be a
/// Location/String mixture — in that case the output is always a String tag
/// (Location values are converted to their sequence representation).
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct FillMissing {
    /// Primary tag: used when not missing.
    #[tpd(alias = "primary")]
    #[tpd(alias = "first")]
    in_label_primary: TagLabel,

    /// Secondary tag: used when the primary tag is missing.
    #[tpd(alias = "secondary")]
    #[tpd(alias = "second")]
    in_label_secondary: TagLabel,

    /// Output tag label for the result.
    out_label: TagLabel,
}

impl VerifyIn<PartialConfig> for PartialFillMissing {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialFillMissing> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let type0 = inner
                .in_label_primary
                .as_ref()
                .and_then(|l| tags_available.get(l))
                .map(|m| &m.tag_type);
            let type1 = inner
                .in_label_secondary
                .as_ref()
                .and_then(|l| tags_available.get(l))
                .map(|m| &m.tag_type);

            let output_type = match (type0, type1) {
                (Some(TagValueType::Location), Some(TagValueType::String))
                | (Some(TagValueType::String), Some(TagValueType::Location)) => {
                    TagValueType::String
                }
                _ => TagValueType::String, //doesn't mater, will get rejected
            };

            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_label_primary
                        .to_used_tag(&[TagValueType::Location, TagValueType::String]),
                    inner
                        .in_label_secondary
                        .to_used_tag(&[TagValueType::Location, TagValueType::String]),
                ],
                declared_tag: inner.out_label.to_declared_tag(output_type),
                must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for FillMissing {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let num_reads = block.segments[0].entries.len();

        let primary_vec = block
            .tags
            .get(&self.in_label_primary)
            .ok_or_else(|| anyhow::anyhow!("Tag '{}' not found in block", self.in_label_primary))?;
        let secondary_vec = block.tags.get(&self.in_label_secondary).ok_or_else(|| {
            anyhow::anyhow!("Tag '{}' not found in block", self.in_label_secondary)
        })?;

        // Detect if we need Location→String conversion (mixed-type pair).
        let force_string = primary_vec.iter().any(|v| matches!(v, TagValue::String(_)))
            || secondary_vec
                .iter()
                .any(|v| matches!(v, TagValue::String(_)));

        let mut output_tags = Vec::with_capacity(num_reads);

        for i in 0..num_reads {
            let chosen = if !matches!(primary_vec[i], TagValue::Missing) {
                &primary_vec[i]
            } else {
                &secondary_vec[i]
            };

            let output = if force_string {
                match chosen {
                    TagValue::Location(hits) => TagValue::String(hits.joined_sequence(None).into()),
                    other => other.clone(),
                }
            } else {
                chosen.clone()
            };

            output_tags.push(output);
        }

        block.tags.insert(self.out_label.clone(), output_tags);

        Ok((block, true))
    }
}
