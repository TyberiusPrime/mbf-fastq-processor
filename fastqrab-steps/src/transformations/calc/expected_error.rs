use super::super::reports::common::{PHRED33OFFSET, Q_LOOKUP};
use crate::transformations::prelude::*;

const PHRED33_MAX: u8 = 126;

#[derive(Clone, Copy, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum ExpectedErrorAggregate {
    Sum,
    Max,
}

/// Calculate expected error from (sanger, 33 based) PHRED
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ExpectedError {
    pub out_label: TagLabel,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,

    pub aggregate: ExpectedErrorAggregate,
}

impl VerifyIn<PartialConfig> for PartialExpectedError {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.or(SegmentIndexOrAll::All);
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialExpectedError> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric((
                    Some(NonNaN::new(0.0).expect("can't fail")),
                    None,
                ))),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for ExpectedError {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let aggregate = self.aggregate;

        let mut result = vec![0.0f64; block.len()];
        for (segment_no, segment) in block.segments.iter().enumerate() {
            for (ii, qual) in segment.seq_quals.iter_qual().enumerate() {
                result[ii] = match expected_error_for_read(qual, aggregate) {
                    Ok(value) => match aggregate {
                        ExpectedErrorAggregate::Sum => value + result[ii],
                        ExpectedErrorAggregate::Max => result[ii].max(value),
                    },
                    Err(err) => {
                        return Err(err.context(format!(
                            "Error calculating expected error for read {} in segment {}",
                            segment.names.get(ii),
                            input_info.segment_order[segment_no]
                        )));
                    }
                };
            }
        }

        block
            .tags
            .insert(self.out_label.clone(), TagColumn::Numeric(result));
        Ok((block, true))
    }
}

fn expected_error_for_read(
    read_quality: &BStr,
    aggregate: ExpectedErrorAggregate,
) -> anyhow::Result<f64> {
    let mut agg = 0.0;

    for &quality in read_quality.iter() {
        if !(PHRED33OFFSET..=PHRED33_MAX).contains(&quality) {
            let quality_display = BString::from(vec![quality]);
            anyhow::bail!(
                "CalcExpectedError requires PHRED+33 encoded qualities (ASCII 33..=126). \
                    Observed byte {quality} ('{}'). \
                    Consider running ConvertQuality before CalcExpectedError.",
                quality_display.escape_ascii(),
            );
        }
        let expected_error = Q_LOOKUP[quality as usize];
        match aggregate {
            ExpectedErrorAggregate::Sum => {
                agg += expected_error;
            }
            ExpectedErrorAggregate::Max => {
                agg = expected_error.max(agg);
            }
        }
    }

    Ok(agg)
}
