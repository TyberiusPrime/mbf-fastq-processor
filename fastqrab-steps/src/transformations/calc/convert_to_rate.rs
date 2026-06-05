use fastqrab_dna::dna::TagColumn;
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
                //must_see_all_tags: true,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ConvertToRate {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let source_values: Vec<f64> = block
            .tags
            .get(&self.in_label)
            .expect("in_label not found - should have been verified at config time")
            .iter_numeric()
            .copied()
            .collect();

        let mut source_iter = source_values.into_iter();

        if let Ok(segment_index) = TryInto::<SegmentIndex>::try_into(self.segment) {
            super::extract_numeric_tags_from_sequences(
                segment_index,
                &self.out_label,
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "loss is acceptable, it's going to be within u32 range"
                )]
                |read| {
                    let source = source_iter
                        .next()
                        .expect("source and segment have same read count");
                    let len = read.len() as f64;
                    if len > 0.0 { source / len } else { 0.0 }
                },
                &mut block,
            );
        } else {
            let len = block.segments[0].len();
            let mut values = vec![0.0; len];
            for segment in &block.segments {
                for ii in 0..len {
                    values[ii] += segment.seq_quals.entry_len(ii) as f64;
                }
            }
            block
                .tags
                .insert(self.out_label.clone(), TagColumn::Numeric(values));
        }

        Ok((block, true))
    }
}
