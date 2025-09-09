#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    Demultiplexed,
    config::{TargetPlusAll, deser::u8_from_char_or_number},
};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct QualifiedBases {
    pub label: String,
    pub target: TargetPlusAll,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
}

impl Step for QualifiedBases {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::super::validate_target_plus_all(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
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
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let qual = read.qual();
                let sum: usize = qual
                    .iter()
                    .map(|x| usize::from(*x >= self.min_quality))
                    .sum();
                sum as f64 / qual.len() as f64
            },
            |read1, read2, index1, index2| {
                let mut sum: usize = 0;
                let mut len = 0;
                for read in Some(read1)
                    .into_iter()
                    .chain(read2.into_iter())
                    .chain(index1.into_iter())
                    .chain(index2.into_iter())
                {
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

        (block, true)
    }
}
