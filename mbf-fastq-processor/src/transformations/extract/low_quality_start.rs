#![allow(clippy::unnecessary_wraps)] //eserde false positives
//
use crate::transformations::prelude::*;

use super::extract_region_tags;
use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::dna::Hits;

/// Turn low quality start's of reads into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct LowQualityStart {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: String,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub min_qual: u8,
}

impl VerifyIn<PartialConfig> for PartialLowQualityStart {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl Step for LowQualityStart {
    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let min_qual = self.min_qual;
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
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
                    self.segment,
                    read.seq()[..cut_pos].to_vec().into(),
                ))
            } else {
                None
            }
        });

        Ok((block, true))
    }
}
