#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::TargetPlusAll, Demultiplexed};

use super::super::Step;
use super::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct GCContent {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for GCContent {
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
        fn gc_count(sequence: &[u8]) -> usize {
            sequence
                .iter()
                .filter(|&&base| base == b'G' || base == b'C' || base == b'g' || base == b'c')
                .count()
        }
        fn non_n_count(sequence: &[u8]) -> usize {
            sequence
                .iter()
                .filter(|&&base| base != b'N' && base != b'n')
                .count()
        }

        extract_numeric_tags_plus_all(
            self.target,
            &self.label,
            |read| {
                let sequence = read.seq();
                if sequence.is_empty() {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        (gc_count(sequence) as f64 / non_n_count(sequence) as f64) * 100.0
                    }
                }
            },
            |read1, read2, index1, index2| {
                let mut total_gc_count = 0usize;
                let mut total_length = 0usize;

                for seq in Some(read1)
                    .into_iter()
                    .chain(read2.into_iter())
                    .chain(index1.into_iter())
                    .chain(index2.into_iter())
                {
                    let sequence = seq.seq();
                    total_gc_count += gc_count(sequence);
                    total_length += non_n_count(sequence);
                }

                if total_length == 0 {
                    0.0
                } else {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        (total_gc_count as f64 / total_length as f64) * 100.0
                    }
                }
            },
            &mut block,
        );

        (block, true)
    }
}