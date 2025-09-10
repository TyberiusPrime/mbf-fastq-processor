#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::SegmentOrAll, Demultiplexed};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Length {
    pub label: String,
    pub segment: SegmentOrAll,
}

impl Step for Length {
    fn validate_segments(
        &mut self,
        input_def: &crate::config::Input,
    ) -> anyhow::Result<()> {
        self.segment.validate(input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn tag_provides_location(&self) -> bool {
        false
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_numeric_tags_plus_all(
            &self.segment,
            &self.label,
            #[allow(clippy::cast_precision_loss)]
            |read| read.seq().len() as f64,
            #[allow(clippy::cast_precision_loss)]
            |reads| {
                let mut total_length: usize = reads.iter().map(|read| read.seq().len()).sum();
                total_length as f64
            },
            &mut block,
        );

        (block, true)
    }
}
