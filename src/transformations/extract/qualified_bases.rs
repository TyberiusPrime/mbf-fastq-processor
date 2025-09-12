#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    Demultiplexed,
    config::{SegmentIndexOrAll, SegmentOrAll, deser::u8_from_char_or_number},
};
use anyhow::Result;

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QualifiedBases {
    pub label: String,

    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
}

impl Step for QualifiedBases {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    #[allow(
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss
    )]
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment_index.as_ref().unwrap(),
            &self.label,
            |read| {
                let qual = read.qual();
                let sum: usize = qual
                    .iter()
                    .map(|x| usize::from(*x >= self.min_quality))
                    .sum();
                sum as f64 / qual.len() as f64
            },
            |reads| {
                let mut sum: usize = 0;
                let mut len = 0;
                for read in reads {
                    let qual = read.qual();
                    sum += qual
                        .iter()
                        .map(|x| usize::from(*x >= self.min_quality))
                        .sum::<usize>();
                    len += qual.len();
                }
                sum as f64 / len as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}
