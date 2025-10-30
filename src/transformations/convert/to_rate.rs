use anyhow::{Result, bail};

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    demultiplex::Demultiplexed,
    dna::TagValue,
    io::{self, WrappedFastQReadMut},
};

use super::super::{Step, TagValueType, Transformation};

#[derive(eserde::Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogBase {
    #[serde(alias = "natural")]
    E,
    #[serde(alias = "2")]
    Two,
    #[serde(alias = "10")]
    Ten,
}

impl LogBase {
    fn apply(self, value: f64, offset: f64) -> f64 {
        let adjusted = value + offset;

        let is_offset_one = (offset - 1.0).abs() <= f64::EPSILON;
        match (self, is_offset_one) {
            (LogBase::E, true) => value.ln_1p(),
            (LogBase::E, false) => adjusted.ln(),
            (LogBase::Two, _) => adjusted.log2(),
            (LogBase::Ten, _) => adjusted.log10(),
        }
    }
}

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ConvertToRate {
    pub label: String,
    #[serde(alias = "numerator_label")]
    pub numerator_label: String,
    #[serde(default)]
    pub denominator_label: Option<String>,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
    #[serde(default)]
    pub log_base: Option<LogBase>,
    #[serde(default)]
    pub log_offset: f64,
}

impl Step for ConvertToRate {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if self.denominator_label.is_none() {
            self.segment_index = Some(self.segment.validate(input_def)?);
        }
        Ok(())
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.log_base.is_none() && self.log_offset != 0.0 {
            bail!("ConvertToRate: 'log_offset' can only be used together with 'log_base'");
        }

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.label.clone(), TagValueType::Numeric))
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        let mut tags: Vec<(String, &[TagValueType])> =
            vec![(self.numerator_label.clone(), &[TagValueType::Numeric])];
        if let Some(denominator_label) = &self.denominator_label {
            tags.push((denominator_label.clone(), &[TagValueType::Numeric]));
        }
        Some(tags)
    }

    fn apply(
        &mut self,
        mut block: io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(io::FastQBlocksCombined, bool)> {
        if block.tags.is_none() {
            bail!(
                "ConvertToRate expects tag '{}' to be available",
                self.numerator_label
            );
        }

        let mut rates: Vec<TagValue> = Vec::with_capacity(block.len());
        let base = self.log_base;
        let offset = self.log_offset;

        let compute_rate = |numerator: f64, denominator: f64| -> f64 {
            if let Some(base) = base {
                let numerator_log = base.apply(numerator, offset);
                let denominator_log = base.apply(denominator, offset);
                numerator_log - denominator_log
            } else {
                numerator / denominator
            }
        };

        if let Some(denominator_label) = &self.denominator_label {
            block.apply_mut_with_tags(
                &self.numerator_label,
                denominator_label,
                |_reads, numerator_tag, denominator_tag| {
                    let numerator = numerator_tag.as_numeric().unwrap();
                    let denominator = denominator_tag.as_numeric().unwrap();
                    rates.push(compute_rate(numerator, denominator).into());
                },
            );
        } else {
            block.apply_mut_with_tag(&self.numerator_label, |reads, numerator_tag| {
                let numerator = numerator_tag.as_numeric().unwrap();

                #[allow(clippy::cast_precision_loss)]
                let denominator = match self.segment_index.unwrap() {
                    SegmentIndexOrAll::Indexed(segment_idx) => reads[segment_idx].len() as f64,
                    SegmentIndexOrAll::All => {
                        reads.iter().map(WrappedFastQReadMut::len).sum::<usize>() as f64
                    }
                };

                rates.push(compute_rate(numerator, denominator).into());
            });
        }

        block
            .tags
            .as_mut()
            .unwrap()
            .insert(self.label.clone(), rates);

        Ok((block, true))
    }
}
