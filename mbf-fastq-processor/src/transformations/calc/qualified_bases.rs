#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::deser::tpd_extract_u8_from_byte_or_char, transformations::prelude::*};

use super::extract_numeric_tags_plus_all;
use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll, deser::u8_from_char_or_number},
    io::WrappedFastQRead,
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub enum Operator {
    #[serde(alias = ">")]
    #[serde(alias = "above")]
    #[serde(alias = "worse")]
    #[serde(alias = "gt")]
    Above,
    #[serde(alias = "<")]
    #[serde(alias = "below")]
    #[serde(alias = "better")]
    #[serde(alias = "lt")]
    Below,
    #[serde(alias = ">=")]
    #[serde(alias = "worse_or_equal")]
    #[serde(alias = "above_or_equal")]
    #[serde(alias = "gte")]
    AboveOrEqual,
    #[serde(alias = "<=")]
    #[serde(alias = "better_or_equal")]
    #[serde(alias = "below_or_equal")]
    #[serde(alias = "lte")]
    BelowOrEqual,
}

/// Calculate bases passing quality threshold (in any direction)
#[derive(Clone, JsonSchema)]
#[tpd(partial = false)]
#[derive(Debug)]
pub struct QualifiedBases {
    pub out_label: String,

    #[tpd_adapt_in_verify]
    pub threshold: u8,

    #[tpd_alias("op")]
    pub operator: Operator,

    #[tpd_default]
    segment: SegmentOrAll,

    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl VerifyFromToml for PartialQualifiedBases {
    fn verify(mut self, _helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.threshold = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_threshold(_helper, false, false),
            self.tpd_get_threshold(_helper, true, false),
        );
        self
    }
}

impl Step for QualifiedBases {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

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
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            one_read,
            |reads| reads.iter().map(one_read).sum(),
            &mut block,
        );

        Ok((block, true))
    }
}
