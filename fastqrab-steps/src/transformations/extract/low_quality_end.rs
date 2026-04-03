use super::extract_region_tags;
use crate::transformations::prelude::*;
use fastqrab_config::{dna::Hits, tpd_adapt_u8_from_byte_or_char};

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

impl Step for LowQualityEnd {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let min_qual = self.min_qual;
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
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
                    self.segment,
                    read.seq()[cut_pos..].to_vec().into(),
                ))
            } else {
                None
            }
        });

        Ok((block, true))
    }
}
