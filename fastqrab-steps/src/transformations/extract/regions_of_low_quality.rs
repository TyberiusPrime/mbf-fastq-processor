use crate::transformations::prelude::*;
use fastqrab_config::tpd_adapt_u8_from_byte_or_char;

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
    pub out_label: TagLabel,
}

impl VerifyIn<PartialConfig> for PartialRegionsOfLowQuality {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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

impl TagUser for PartialTaggedVariant<PartialRegionsOfLowQuality> {
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

impl Step for RegionsOfLowQuality {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut col = LocationColumn::new();
        let segment = self.segment;
        let min_quality = self.min_quality;
        let min_length = self.min_length;
        let f = |read: &mut fastqrab_io::io::WrappedFastQRead| {
            let quality_scores = read.qual();
            let mut entries: Vec<(Option<HitRegionView>, Vec<u8>)> = Vec::new();
            let mut in_low_quality_region = false;
            let mut region_start = 0;

            for (pos, &qual) in quality_scores.iter().enumerate() {
                let is_low_quality = qual < min_quality;

                if is_low_quality && !in_low_quality_region {
                    in_low_quality_region = true;
                    region_start = pos;
                } else if !is_low_quality && in_low_quality_region {
                    in_low_quality_region = false;
                    let region_len = pos - region_start;
                    if region_len >= min_length {
                        entries.push((
                            Some(HitRegionView {
                                segment_index: segment,
                                start: region_start,
                                len: region_len,
                            }),
                            read.seq()[region_start..pos].to_vec(),
                        ));
                    }
                }
            }

            if in_low_quality_region {
                let region_len = quality_scores.len() - region_start;
                if region_len >= min_length {
                    entries.push((
                        Some(HitRegionView {
                            segment_index: segment,
                            start: region_start,
                            len: region_len,
                        }),
                        read.seq()[region_start..].to_vec(),
                    ));
                }
            }

            if entries.is_empty() {
                col.push_none();
            } else {
                let refs: Vec<(Option<HitRegionView>, &[u8])> = entries
                    .iter()
                    .map(|(loc, seq)| (loc.clone(), seq.as_slice()))
                    .collect();
                col.push_many(&refs);
            }
        };
        block.segments[self.segment.as_index()].apply(f);
        block
            .tags
            .insert(self.out_label.clone(), TagColumn::Location(col));

        Ok((block, true))
    }
}
