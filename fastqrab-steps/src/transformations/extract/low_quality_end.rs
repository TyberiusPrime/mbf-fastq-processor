use crate::transformations::{extract::extract_region_tags_from_qual, prelude::*};
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;

/// Turn low quality end's of reads into a tag
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct LowQualityEnd {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: TagLabel,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub min_qual: u8,
}

impl VerifyIn<PartialConfig> for PartialLowQualityEnd {
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

impl TagUser for PartialTaggedVariant<PartialLowQualityEnd> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut()
            && let Some(Some(segment)) = inner.segment.as_ref().map(|m| m.as_ref_post())
        {
            // Record the segment this location tag lives on, so a conditional
            // `Swap` on that segment can forget it (a per-read swap would split it
            // across two segments).
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Location)
                    .map(|dt| dt.with_segment(*segment)),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for LowQualityEnd {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let min_qual = self.min_qual;
        extract_region_tags_from_qual(&mut block, self.segment, &self.out_label, |qual| {
            let mut cut_pos = qual.len();
            for q in qual.iter().rev() {
                if *q < min_qual {
                    cut_pos -= 1;
                } else {
                    break;
                }
            }
            #[expect(
                clippy::cast_possible_truncation,
                reason = "lengths are guaranteed to be within u32 range"
            )]
            Some(cut_pos as u32..qual.len() as u32)
        });

        Ok((block, true))
    }
}
