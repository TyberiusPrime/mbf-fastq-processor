#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use super::extract_region_tags;
use crate::{
    config::deser::tpd_adapt_u8_from_byte_or_char,
    dna::{Hit, HitRegion, Hits},
};

/// Extract regions of low quality (configurable)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct RegionsOfLowQuality {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub min_quality: u8,
    pub min_length: usize,
    pub out_label: String,
}

impl VerifyIn<PartialConfig> for PartialRegionsOfLowQuality {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);

        self.min_length.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "Must be > 0",
                    Some("Change to a positive integer"),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl Step for RegionsOfLowQuality {
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
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
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
                                segment_index: self.segment,
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
                            segment_index: self.segment,
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
        });

        Ok((block, true))
    }
}
