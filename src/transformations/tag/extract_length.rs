#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::TargetPlusAll, Demultiplexed};

use super::super::Step;
use super::common::extract_numeric_tags_plus_all;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtractLength {
    pub label: String,
    pub target: TargetPlusAll,
}

impl Step for ExtractLength {
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
            #[allow(clippy::cast_precision_loss)]
            |read| read.seq().len() as f64,
            #[allow(clippy::cast_precision_loss)]
            |read1, read2, index1, index2| {
                let mut total_length = read1.seq().len();
                if let Some(read2) = read2 {
                    total_length += read2.seq().len();
                }
                if let Some(index1) = index1 {
                    total_length += index1.seq().len();
                }
                if let Some(index2) = index2 {
                    total_length += index2.seq().len();
                }
                total_length as f64
            },
            &mut block,
        );

        (block, true)
    }
}