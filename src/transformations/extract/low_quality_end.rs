#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{Step, Target, Transformation, validate_target};
use super::extract_tags;
use crate::dna::Hits;
use crate::{config::deser::u8_from_char_or_number, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowQualityEnd {
    pub target: Target,
    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_qual: u8,
}

impl Step for LowQualityEnd {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let min_qual = self.min_qual;
        extract_tags(&mut block, self.target, &self.label, |read| {
            let qual = read.qual();
            let mut cut_pos = qual.len();
            for q in qual.iter().rev() {
                if *q < min_qual {
                    cut_pos -= 1;
                } else {
                    break;
                }
            }
            if cut_pos < qual.len() {
                Some(Hits::new(
                    cut_pos,
                    qual.len() - cut_pos,
                    self.target,
                    read.seq()[cut_pos..].to_vec().into(),
                ))
            } else {
                None
            }
        });

        (block, true)
    }
}
