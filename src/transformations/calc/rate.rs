use anyhow::{anyhow, bail, Context, Result};

use crate::{
    config::{SegmentIndexOrAll, SegmentOrAll},
    demultiplex::Demultiplexed,
    dna::TagValue,
    io,
};

use super::super::{filters, Step, TagValueType, Transformation};

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
pub struct CalcRate {
    pub label: String,
    #[serde(alias = "numerator_label")]
    pub nominator_label: String,
    #[serde(alias = "numerator")]
    #[serde(alias = "nominator")]
    #[serde(alias = "nominator_tag")]
    #[serde(alias = "numerator_tag")]
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

impl Step for CalcRate {
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
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        filters::validate_tag_set_and_type(
            all_transforms,
            this_transforms_index,
            &self.nominator_label,
            TagValueType::Numeric,
        )?;

        if let Some(denominator_label) = &self.denominator_label {
            filters::validate_tag_set_and_type(
                all_transforms,
                this_transforms_index,
                denominator_label,
                TagValueType::Numeric,
            )?;
        }

        if self.log_base.is_none() && self.log_offset != 0.0 {
            bail!("CalcRate: 'log_offset' can only be used together with 'log_base'");
        }

        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, TagValueType)> {
        Some((self.label.clone(), TagValueType::Numeric))
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        let mut tags = vec![self.nominator_label.clone()];
        if let Some(denominator_label) = &self.denominator_label {
            tags.push(denominator_label.clone());
        }
        Some(tags)
    }

    fn tag_provides_location(&self) -> bool {
        false
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
                "CalcRate expects tag '{}' to be available",
                self.nominator_label
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
                &self.nominator_label,
                denominator_label,
                |_reads, numerator_tag, denominator_tag| {
                    let numerator = numerator_tag.as_numeric().unwrap();
                    let denominator = denominator_tag.as_numeric().unwrap();
                    rates.push(compute_rate(numerator, denominator).into());
                },
            );
        } else {
            block.apply_mut_with_tag(&self.nominator_label, |reads, numerator_tag| {
                let numerator = numerator_tag.as_numeric().unwrap();

                #[allow(clippy::cast_precision_loss)]
                let denominator = match self.segment_index.unwrap() {
                    SegmentIndexOrAll::Indexed(segment_idx) => reads[segment_idx].len() as f64,
                    SegmentIndexOrAll::All => {
                        reads.iter().map(|read| read.len()).sum::<usize>() as f64
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
