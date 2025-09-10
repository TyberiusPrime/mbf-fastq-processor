#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use std::{collections::HashSet, path::Path};

use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use crate::transformations::{
    reproducible_cuckoofilter, FragmentEntry, InputInfo, Segment, Step, Transformation,
};
use serde_valid::Validate;

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;

#[derive(eserde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct OtherFileBySequence {
    pub filename: String,
    #[eserde(compat)]
    pub segment: Segment,
    pub label: String,

    pub seed: u64,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,

    pub ignore_unaligned: Option<bool>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,
}

impl Step for OtherFileBySequence {
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn validate_others(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if (self.filename.ends_with(".bam") || self.filename.ends_with(".sam"))
            && self.ignore_unaligned.is_none()
        {
            return Err(anyhow::anyhow!(
                "When using a BAM file, you must specify `ignore_unaligned` = true|false"
            ));
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment.validate(input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        let mut filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
            ApproxOrExactFilter::Exact(HashSet::new())
        } else {
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                self.seed,
                100_000,
                self.false_positive_rate,
            )))
        };
        // read them all.
        crate::io::apply_to_read_sequences(
            &self.filename,
            &mut |read_seq| {
                filter.insert(&FragmentEntry(&[read_seq]));
            },
            self.ignore_unaligned,
        )?;
        self.filter = Some(filter);
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_bool_tags(&mut block, &self.segment, &self.label, |read| {
            let filter = self.filter.as_ref().unwrap();
            let query = read.seq();
            filter.contains(&FragmentEntry(&[query]))
        });
        (block, true)
    }
}
