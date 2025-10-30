#![allow(clippy::unnecessary_wraps)]
use crate::config::SegmentIndexOrAll;
use anyhow::Result;
//eserde false positives
use crate::{Demultiplex, config::SegmentOrAll};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Length {
    pub label: String,
    #[serde(default)]
    pub segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
}

impl Step for Length {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Numeric,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        extract_numeric_tags_plus_all(
            self.segment_index.unwrap(),
            &self.label,
            #[allow(clippy::cast_precision_loss)]
            |read| read.seq().len() as f64,
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let total_length: usize = reads.iter().map(|read| read.seq().len()).sum();
                total_length as f64
            },
            &mut block,
        );

        Ok((block, true))
    }
}
