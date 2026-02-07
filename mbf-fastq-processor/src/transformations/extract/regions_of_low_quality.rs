#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::{config::deser::tpd_extract_u8_from_byte_or_char, transformations::prelude::*};

use super::extract_region_tags;
use crate::{
    config::deser::u8_from_char_or_number,
    dna::{Hit, HitRegion, Hits},
};

/// Extract regions of low quality (configurable)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct RegionsOfLowQuality {
    #[tpd_default]
    segment: Segment,
    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndex>,

    #[tpd_adapt_in_verify]
    pub min_quality: u8,
    pub min_length: usize,
    pub out_label: String,
}

impl VerifyFromToml for PartialRegionsOfLowQuality {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.min_quality = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_min_quality(helper, false), // one required check is enough.
            self.tpd_get_min_quality(helper, true),
        );
        self
    }
}

impl Step for RegionsOfLowQuality {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.min_length == 0 {
            bail!("min_length must be > 0 in RegionsOfLowQuality. Change to a positive integer.");
        }
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
        extract_region_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read| {
                let quality_scores = read.qual();
                let mut regions = Vec::new();
                let mut in_low_quality_region = false;
                let mut region_start = 0;

                for (pos, &qual) in quality_scores.iter().enumerate() {
                    let is_low_quality = qual < self.min_quality;

                    if is_low_quality && !in_low_quality_region {
                        // Start of a new low quality region
                        in_low_quality_region = true;
                        region_start = pos;
                    } else if !is_low_quality && in_low_quality_region {
                        // End of low quality region
                        in_low_quality_region = false;
                        let region_len = pos - region_start;
                        if region_len >= self.min_length {
                            regions.push(Hit {
                                location: Some(HitRegion {
                                    segment_index: self
                                        .segment_index
                                        .expect("segment_index must be set during initialization"),
                                    start: region_start,
                                    len: region_len,
                                }),
                                sequence: read.seq()[region_start..pos].into(),
                            });
                        }
                    }
                }

                // Handle case where sequence ends with low quality region
                if in_low_quality_region {
                    let region_len = quality_scores.len() - region_start;
                    if region_len >= self.min_length {
                        regions.push(Hit {
                            location: Some(HitRegion {
                                segment_index: self
                                    .segment_index
                                    .expect("segment_index must be set during initialization"),
                                start: region_start,
                                len: region_len,
                            }),
                            sequence: read.seq()[region_start..].into(),
                        });
                    }
                }

                if regions.is_empty() {
                    None
                } else {
                    Some(Hits::new_multiple(regions))
                }
            },
        );

        Ok((block, true))
    }
}
