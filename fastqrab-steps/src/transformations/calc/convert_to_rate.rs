use toml_pretty_deser::PartialTaggedVariant;

use crate::transformations::prelude::*;

/// Divide a numeric tag value by the read length to produce a rate (0..=1)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ConvertToRate {
    pub in_label: TagLabel,
    pub out_label: TagLabel,
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialConvertToRate {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialConvertToRate> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                used_tags: vec![
                    inner
                        .in_label
                        .to_used_tag(&[TagValueType::Numeric((None, None))][..]),
                ],
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric((
                    Some(NonNaN::new(0.0).expect("can't fail")),
                    Some(NonNaN::new(1.0).expect("can't fail")),
                ))),
                must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl Step for ConvertToRate {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let source_values: Vec<f64> = block
            .tags
            .get(&self.in_label)
            .expect("in_label not found - should have been verified at config time")
            .iter()
            .map(|v| {
                v.as_numeric()
                    .expect("in_label should be numeric - verified at config time")
            })
            .collect();

        let mut source_iter = source_values.into_iter();

        if let Ok(segment_index) = TryInto::<SegmentIndex>::try_into(self.segment) {
            super::extract_numeric_tags(
                segment_index,
                &self.out_label,
                #[allow(clippy::cast_precision_loss)]
                |read| {
                    let source = source_iter
                        .next()
                        .expect("source and segment have same read count");
                    let len = read.seq().len() as f64;
                    if len > 0.0 { source / len } else { 0.0 }
                },
                &mut block,
            );
        } else {
            let mut values = Vec::with_capacity(block.segments[0].len());
            let mut block_iter = block.get_pseudo_iter();
            while let Some(molecule) = block_iter.pseudo_next() {
                let source = source_iter
                    .next()
                    .expect("source and segments have same read count");
                #[allow(clippy::cast_precision_loss)]
                let total_len = molecule
                    .segments
                    .iter()
                    .map(|r| r.seq().len())
                    .sum::<usize>() as f64;
                values.push(TagValue::Numeric(if total_len > 0.0 {
                    source / total_len
                } else {
                    0.0
                }));
            }
            block.tags.insert(self.out_label.clone(), values);
        }

        Ok((block, true))
    }
}