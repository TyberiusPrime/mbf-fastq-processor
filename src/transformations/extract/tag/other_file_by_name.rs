#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use std::cell::Cell;
use std::{collections::HashSet, path::Path};

use crate::config::{Segment, SegmentIndex};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use crate::transformations::{
    FragmentEntry, InputInfo, Step, Transformation, reproducible_cuckoofilter,
};
use serde_valid::Validate;

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::config::deser::single_u8_from_string;
use crate::transformations::read_name_canonical_prefix;
use crate::transformations::tag::initial_filter_elements;

#[derive(eserde::Deserialize, Debug, Validate, Clone)]
#[serde(deny_unknown_fields)]
pub struct OtherFileByName {
    pub filename: String,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub label: String,
    pub seed: Option<u64>,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,

    pub ignore_unaligned: Option<bool>,

    #[serde(default, deserialize_with = "single_u8_from_string")]
    pub fastq_readname_end_char: Option<u8>,

    #[serde(default, deserialize_with = "single_u8_from_string")]
    pub reference_readname_end_char: Option<u8>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<ApproxOrExactFilter>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl Step for OtherFileByName {
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
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

        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn store_progress_output(&mut self, _progress: &crate::transformations::reports::Progress) {
        self.progress_output = Some(_progress.clone());
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
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
            let seed = self
                .seed
                .expect("seed should be validated to exist when false_positive_rate > 0.0");
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                seed,
                initial_filter_elements(&self.filename),
                self.false_positive_rate,
            )))
        };
        // read them all.
        crate::io::apply_to_read_names(
            &self.filename,
            &mut |read_name| {
                let trimmed =
                    read_name_canonical_prefix(read_name, self.reference_readname_end_char);

                if !filter.contains(&FragmentEntry(&[trimmed])) {
                    filter.insert(&FragmentEntry(&[trimmed]));
                }
            },
            self.ignore_unaligned,
        )?;
        self.filter = Some(filter);
        Ok(None)
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if let Some(pg) = self.progress_output.as_mut() {
            pg.output(&format!("Reading all read names from {}", self.filename));
        }
        let count: Cell<usize> = Cell::new(0);
        extract_bool_tags(
            &mut block,
            self.segment_index.unwrap(),
            &self.label,
            |read| {
                count.set(count.get() + 1);
                let query = read_name_canonical_prefix(read.name(), self.fastq_readname_end_char);

                self.filter
                    .as_ref()
                    .unwrap()
                    .contains(&FragmentEntry(&[query]))
            },
        );
        if let Some(pg) = self.progress_output.as_mut() {
            pg.output(&format!(
                "Finished reading all ({}) read names from {}",
                count.get(),
                self.filename
            ));
        }

        Ok((block, true))
    }
}
