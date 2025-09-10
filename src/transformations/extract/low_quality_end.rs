#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{Step, Segment, Transformation};
use super::extract_tags;
use crate::dna::Hits;
use crate::{config::deser::u8_from_char_or_number, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowQualityEnd {
    pub segment: Segment,
    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_qual: u8,
}

impl Step for LowQualityEnd {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> Result<()> {
        self.segment.validate(input_def)
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
        extract_tags(&mut block, &self.segment, &self.label, |read| {
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
                    self.segment.clone(),
                    read.seq()[cut_pos..].to_vec().into(),
                ))
            } else {
                None
            }
        });

        (block, true)
    }
}
