#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::extract_numeric_tags_plus_all;
use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll, deser::u8_from_char_or_number},
    io::WrappedFastQRead,
};

#[repr(u8)]
#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum Operator {
    #[serde(alias = ">")]
    #[serde(alias = "Above")]
    #[serde(alias = "above")]
    #[serde(alias = "worse")]
    #[serde(alias = "Worse")]
    #[serde(alias = "gt")]
    Above,
    #[serde(alias = "<")]
    #[serde(alias = "Below")]
    #[serde(alias = "below")]
    #[serde(alias = "better")]
    #[serde(alias = "Better")]
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

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QualifiedBases {
    pub out_label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub threshold: u8,
    #[serde(alias = "op")]
    pub operator: Operator,

    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
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
