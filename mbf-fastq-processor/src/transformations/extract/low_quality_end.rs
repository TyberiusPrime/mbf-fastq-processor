#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::extract_region_tags;
use crate::config::deser::tpd_extract_u8_from_byte_or_char;
use crate::dna::Hits;

/// Turn low quality end's of reads into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct LowQualityEnd {
    #[tpd_default]
    segment: Segment,
    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,
    #[tpd_adapt_in_verify]
    pub min_qual: u8,
}


impl VerifyFromToml for PartialLowQualityEnd {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.min_qual = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_min_qual(helper, false, false), // one required check is enough.
            self.tpd_get_min_qual(helper, true, false),
        );
        self
    }
}

impl Step for LowQualityEnd {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

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
        extract_region_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
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
                        self.segment_index
                            .expect("segment_index must be set during initialization"),
                        read.seq()[cut_pos..].to_vec().into(),
                    ))
                } else {
                    None
                }
            },
        );

        Ok((block, true))
    }
}
