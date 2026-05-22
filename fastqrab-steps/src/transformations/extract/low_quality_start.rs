use super::extract_region_tags;
use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;

/// Turn low quality start's of reads into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct LowQualityStart {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: TagLabel,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub min_qual: u8,
}

impl VerifyIn<PartialConfig> for PartialLowQualityStart {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialLowQualityStart> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Location),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for LowQualityStart {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
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
                Some(HitDraft {
                    location: Some(HitRegionView {
                        start: 0,
                        len: cut_pos,
                        segment_index: self.segment,
                    }),
                    sequence: read.seq()[..cut_pos].to_vec(),
                })
            } else {
                None
            }
        });

        Ok((block, true))
    }
}
