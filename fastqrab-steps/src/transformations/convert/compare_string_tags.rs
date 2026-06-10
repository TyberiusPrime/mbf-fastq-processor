use crate::transformations::prelude::*;

/// Compare two string- or location-valued tags lexicographically (byte-by-byte).
///
/// Returns -1.0 if tag_a < tag_b, 0.0 if equal, 1.0 if tag_a > tag_b,
/// like Rust's `PartialOrd` on byte slices. Missing on either input → Missing output.
/// Raises a runtime error if the two byte sequences for a read have different lengths.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct CompareStringTags {
    pub in_label_a: TagLabel,
    pub in_label_b: TagLabel,
    pub out_label: TagLabel,
}

impl VerifyIn<PartialConfig> for PartialCompareStringTags {
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

impl TagUser for PartialTaggedVariant<PartialCompareStringTags> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_label_a
                        .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                    inner
                        .in_label_b
                        .to_used_tag(&[TagValueType::String, TagValueType::Location]),
                ],
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric((
                    Some(NonNaN::new(-1.0).expect("constant")),
                    Some(NonNaN::new(1.0).expect("constant")),
                ))),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for CompareStringTags {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let col_a = block
            .tags
            .get(&self.in_label_a)
            .expect("in_label_a not found - should have been verified at config time");
        let col_b = block
            .tags
            .get(&self.in_label_b)
            .expect("in_label_b not found - should have been verified at config time");

        let mut results: Vec<f64> = Vec::with_capacity(col_a.len());
        for (idx, (sa, sb)) in col_a
            .iter_stringified()
            .zip(col_b.iter_stringified())
            .enumerate()
        {
            if sa.len() != sb.len() {
                let read_name = BStr::new(block.segments[0].names.get(idx));
                anyhow::bail!(
                    "CompareStringTags requires identical length tags.\n\
                             Read '{read_name}': '{}' has length {} but '{}' has length {}.",
                    self.in_label_a,
                    sa.len(),
                    self.in_label_b,
                    sb.len()
                );
            }
            let result = match sa.as_ref().cmp(sb.as_ref()) {
                std::cmp::Ordering::Less => -1.0,
                std::cmp::Ordering::Equal => 0.0,
                std::cmp::Ordering::Greater => 1.0,
            };
            results.push(result);
        }

        block
            .tags
            .insert(self.out_label.clone(), TagColumn::Numeric(results));
        Ok((block, true))
    }
}
