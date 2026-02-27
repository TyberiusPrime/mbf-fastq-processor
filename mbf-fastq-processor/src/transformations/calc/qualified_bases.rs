#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::extract_numeric_tags_plus_all;
use crate::{
    config::{SegmentIndexOrAll, deser::tpd_adapt_u8_from_byte_or_char},
    io::WrappedFastQRead,
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum Operator {
    #[tpd(alias = ">")]
    #[tpd(alias = "Above")]
    #[tpd(alias = "Worse")]
    #[tpd(alias = "gt")]
    Above,
    #[tpd(alias = "<")]
    #[tpd(alias = "Below")]
    #[tpd(alias = "Better")]
    #[tpd(alias = "lt")]
    Below,
    #[tpd(alias = ">=")]
    #[tpd(alias = "Worse_or_equal")]
    #[tpd(alias = "Wbove_or_equal")]
    #[tpd(alias = "gte")]
    AboveOrEqual,
    #[tpd(alias = "<=")]
    #[tpd(alias = "Better_or_equal")]
    #[tpd(alias = "Below_or_equal")]
    #[tpd(alias = "lte")]
    BelowOrEqual,
}

/// Calculate bases passing quality threshold (in any direction)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct QualifiedBases {
    pub out_label: String,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub threshold: u8,

    #[tpd(alias = "op")]
    pub operator: Operator,

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

impl Step for QualifiedBases {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let op = self.operator;
        let threshold = self.threshold;
        let one_read = |read: &WrappedFastQRead| {
            let it = read.qual().iter();
            let count: usize = match op {
                Operator::Above => it.map(|x| usize::from(*x > threshold)).sum(),
                Operator::Below => it.map(|x| usize::from(*x < threshold)).sum(),
                Operator::AboveOrEqual => it.map(|x| usize::from(*x >= threshold)).sum(),
                Operator::BelowOrEqual => it.map(|x| usize::from(*x <= threshold)).sum(),
            };
            count as f64
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
