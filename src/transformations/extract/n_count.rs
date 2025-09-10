#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::SegmentOrAll, Demultiplexed};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NCount {
    pub label: String,
    pub segment: SegmentOrAll,
}

impl Step for NCount {
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
            |reads| { //todo: fold into one function
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
