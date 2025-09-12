#![allow(clippy::unnecessary_wraps)]
use crate::config::SegmentIndexOrAll;
use anyhow::Result;
//eserde false positives
use crate::{Demultiplexed, config::SegmentOrAll};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NCount {
    pub label: String,
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,
}

impl Step for NCount {
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

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            &self.segment_index.as_ref().unwrap(),
            &self.label,
            |read| {
                let sequence = read.seq();
                #[allow(clippy::cast_precision_loss)]
                {
                    sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count() as f64
                }
            },
            |reads| {
                //todo: fold into one function
                let mut total_n_count = 0usize;

                // Process read1
                for read in reads {
                    let sequence = read.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                #[allow(clippy::cast_precision_loss)]
                {
                    total_n_count as f64
                }
            },
            &mut block,
        );

        (block, true)
    }
}
