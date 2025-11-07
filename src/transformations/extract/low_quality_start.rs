#![allow(clippy::unnecessary_wraps)] //eserde false positives
//
use crate::transformations::prelude::*;

use super::extract_region_tags;
use crate::config::deser::u8_from_char_or_number;
use crate::dna::Hits;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LowQualityStart {
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub label: String,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_qual: u8,
}

impl Step for LowQualityStart {
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
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let min_qual = self.min_qual;
        extract_region_tags(
            &mut block,
            self.segment_index.unwrap(),
            &self.label,
            |read| {
                let mut cut_pos = 0;
                let qual = read.qual();
                for (ii, q) in qual.iter().enumerate() {
                    if *q < min_qual {
                        cut_pos = ii + 1;
                    } else {
                        break;
                    }
                }
                if cut_pos > 0 {
                    Some(Hits::new(
                        0,
                        cut_pos,
                        self.segment_index.unwrap(),
                        read.seq()[..cut_pos].to_vec().into(),
                    ))
                } else {
                    None
                }
            },
        );

        Ok((block, true))
    }
}
