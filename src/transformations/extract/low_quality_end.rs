#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use super::extract_tags;
use crate::config::{Segment, SegmentIndex};
use crate::dna::Hits;
use crate::{config::deser::u8_from_char_or_number, demultiplex::Demultiplexed};
use anyhow::Result;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct LowQualityEnd {
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_qual: u8,
}

impl Step for LowQualityEnd {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let min_qual = self.min_qual;
        extract_tags(
            &mut block,
            self.segment_index.as_ref().unwrap(),
            &self.label,
            |read| {
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
                        self.segment_index.as_ref().unwrap().clone(),
                        read.seq()[cut_pos..].to_vec().into(),
                    ))
                } else {
                    None
                }
            },
        );

        (block, true)
    }
}
