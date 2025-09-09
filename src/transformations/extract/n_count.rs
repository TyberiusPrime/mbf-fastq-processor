#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{Demultiplexed, config::TargetPlusAll};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct NCount {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for NCount {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::super::validate_target_plus_all(self.target, input_def)
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
            self.target,
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
            |read1, read2, index1, index2| {
                let mut total_n_count = 0usize;

                // Process read1
                let sequence = read1.seq();
                total_n_count += sequence
                    .iter()
                    .filter(|&&base| base == b'N' || base == b'n')
                    .count();

                // Process read2 if present
                if let Some(read2) = read2 {
                    let sequence = read2.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                // Process index1 if present
                if let Some(index1) = index1 {
                    let sequence = index1.seq();
                    total_n_count += sequence
                        .iter()
                        .filter(|&&base| base == b'N' || base == b'n')
                        .count();
                }

                // Process index2 if present
                if let Some(index2) = index2 {
                    let sequence = index2.seq();
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
