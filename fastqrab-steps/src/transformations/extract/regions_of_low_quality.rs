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
            // Record the segment this location tag lives on, so a conditional
            // `Swap` on that segment can forget it (a per-read swap would split it
            // across two segments).
            let segment = inner
                .segment
                .as_ref()
                .and_then(|m| m.as_ref_post())
                .copied();
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Location)
                    .map(|dt| match segment {
                        Some(seg) => dt.with_segment(seg),
                        None => dt,
                    }),
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
        let segment = self.segment;
        let mut col = block.location_column_builder(segment);
        let min_quality = self.min_quality;
        let min_length = self.min_length;
        for read in &block.member(self.segment.as_index()).seq_quals {
            let mut entries: Vec<(u32, u32)> = Vec::new();
            let mut in_low_quality_region = false;
            let mut region_start = 0;

            for (pos, &qual) in read.qual.iter().enumerate() {
                let is_low_quality = qual < min_quality;

                if is_low_quality && !in_low_quality_region {
                    in_low_quality_region = true;
                    region_start = pos;
                } else if !is_low_quality && in_low_quality_region {
                    in_low_quality_region = false;
                    let region_len = pos - region_start;
                    if region_len >= min_length {
                        entries.push((
                            region_start.try_into().expect("region_start exceeds u32"),
                            region_len.try_into().expect("Region_len exceeds u32"),
                        ));
                    }
                }
            }

            if in_low_quality_region {
                let region_len = read.qual.len() - region_start;
                if region_len >= min_length {
                    entries.push((
                        region_start.try_into().expect("region_start exceeds u32"),
                        region_len.try_into().expect("Region_len exceeds u32"),
                    ));
                }
            }
            col.push_row(&entries);
        }
        block
            .tags
            .insert(self.out_label.clone(), TagColumn::Location(col.finish()));

        Ok((block, true))
    }
}
