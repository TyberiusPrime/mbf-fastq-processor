use super::extract_numeric_tags_plus_all;
use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;
use fastqrab_io::io::WrappedFastQRead;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum Operator {
    #[tpd(alias = ">")]
    #[tpd(alias = "Worse")]
    #[tpd(alias = "gt")]
    Above,
    #[tpd(alias = "<")]
    #[tpd(alias = "Better")]
    #[tpd(alias = "lt")]
    Below,
    #[tpd(alias = ">=")]
    #[tpd(alias = "WorseOrEqual")]
    #[tpd(alias = "gte")]
    AboveOrEqual,
    #[tpd(alias = "<=")]
    #[tpd(alias = "BetterOrEqual")]
    #[tpd(alias = "lte")]
    BelowOrEqual,
}

/// Calculate bases passing quality threshold (in any direction)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct QualifiedBases {
    pub out_label: TagLabel,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub threshold: u8,

    #[tpd(alias = "op")]
    pub operator: Operator,

    #[tpd(alias = "rate")]
    pub relative: bool,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for PartialQualifiedBases {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialQualifiedBases> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Numeric(
                    if inner.relative.as_ref().is_some_and(|x| *x) {
                        (
                            Some(NonNaN::new(0.0).expect("can't fail")),
                            Some(NonNaN::new(1.0).expect("can't fail")),
                        )
                    } else {
                        (None, None)
                    },
                )),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for QualifiedBases {
    #[expect(
        clippy::cast_precision_loss,
        reason="loss is acceptable, it's going to be within u32 range")]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let op = self.operator;
        let relative = self.relative;
        let threshold = self.threshold;
        let one_read = |read: &WrappedFastQRead| {
            let it = read.qual().iter();
            let count: usize = match op {
                Operator::Above => it.map(|x| usize::from(*x > threshold)).sum(),
                Operator::Below => it.map(|x| usize::from(*x < threshold)).sum(),
                Operator::AboveOrEqual => it.map(|x| usize::from(*x >= threshold)).sum(),
                Operator::BelowOrEqual => it.map(|x| usize::from(*x <= threshold)).sum(),
            };
            if relative {
                count as f64 / read.len() as f64
            } else {
                count as f64
            }
        };

        extract_numeric_tags_plus_all(
            self.segment,
            &self.out_label,
            one_read,
            |reads| reads.iter().map(one_read).sum(),
            &mut block,
        );

        Ok((block, true))
    }
}
