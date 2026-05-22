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

fn tag_to_bytes(tag: &TagValue) -> Option<Vec<u8>> {
    match tag {
        TagValue::String(s) => Some(s.to_vec()),
        TagValue::Location(hits) => Some(hits.joined_sequence(None)),
        TagValue::Missing => None,
        _ => unreachable!("CompareStringTags: unexpected tag type (should be validated)"),
    }
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
        let tags_a = block
            .tags
            .get(&self.in_label_a)
            .expect("in_label_a not found - should have been verified at config time")
            .clone();
        let tags_b = block
            .tags
            .get(&self.in_label_b)
            .expect("in_label_b not found - should have been verified at config time")
            .clone();

        let mut results: Vec<TagValue> = Vec::with_capacity(tags_a.len());
        for (idx, (a, b)) in tags_a.iter().zip(tags_b.iter()).enumerate() {
            let result = match (tag_to_bytes(a), tag_to_bytes(b)) {
                (Some(sa), Some(sb)) => {
                    if sa.len() != sb.len() {
                        let read_name = BStr::new(
                            block.segments[0].entries[idx]
                                .name
                                .get(&block.segments[0].block),
                        );
                        anyhow::bail!(
                            "CompareStringTags requires identical length tags.\n\
                             Read '{read_name}': '{}' has length {} but '{}' has length {}.",
                            self.in_label_a,
                            sa.len(),
                            self.in_label_b,
                            sb.len()
                        );
                    }
                    match sa.cmp(&sb) {
                        std::cmp::Ordering::Less => TagValue::Numeric(-1.0),
                        std::cmp::Ordering::Equal => TagValue::Numeric(0.0),
                        std::cmp::Ordering::Greater => TagValue::Numeric(1.0),
                    }
                }
                _ => TagValue::Numeric(f64::NAN),
            };
            results.push(result);
        }

        block.tags.insert(self.out_label.clone(), results);
        Ok((block, true))
    }
}
