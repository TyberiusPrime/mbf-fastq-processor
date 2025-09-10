#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    Demultiplexed,
    config::{Segment, deser::u8_from_char_or_number},
    dna::{Hit, HitRegion, Hits},
};

use super::super::Step;
use super::extract_tags;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RegionsOfLowQuality {
    pub segment: Segment,
    #[serde(deserialize_with = "u8_from_char_or_number")]
    pub min_quality: u8,
    pub label: String,
}

impl Step for RegionsOfLowQuality {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> anyhow::Result<()> {
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
        extract_tags(&mut block, &self.segment, &self.label, |read| {
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
                    if region_len > 0 {
                        regions.push(Hit {
                            location: Some(HitRegion {
                                segment: self.segment.clone(),
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
                if region_len > 0 {
                    regions.push(Hit {
                        location: Some(HitRegion {
                            segment: self.segment.clone(),
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

        (block, true)
    }
}
