#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use std::cell::Cell;
use std::sync::Arc;
use std::{collections::HashSet, path::Path};

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::transformations::tag::initial_filter_elements;
use crate::transformations::{FragmentEntry, InputInfo, reproducible_cuckoofilter};

/// Tag whether reads are in another file (by sequence)
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtherFileBySequence {
    pub filename: String,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,

    pub seed: Option<u64>,
    pub false_positive_rate: f64,

    pub include_mapped: Option<bool>,
    pub include_unmapped: Option<bool>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl Step for OtherFileBySequence {
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    #[allow(clippy::case_sensitive_file_extension_comparisons)] //sorry, but .BAM is wrong :).
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        if self.filename.ends_with(".bam") || self.filename.ends_with(".sam") {
            if self.include_unmapped.is_none() {
                return Err(anyhow::anyhow!(
                    "When using a BAM file, you must specify `include_unmapped` = true|false"
                ));
            }

            if self.include_mapped.is_none() {
                return Err(anyhow::anyhow!(
                    "When using a BAM file, you must specify `include_mapped` = true|false"
                ));
            }
        }

        self.include_mapped = self.include_mapped.or(Some(false)); // just so it's always set.
        self.include_unmapped = self.include_unmapped.or(Some(false));
        if self.filename.ends_with(".bam") || self.filename.ends_with(".sam") {
            if let (false, false) = (
                self.include_mapped.expect("Just set above"),
                self.include_unmapped.expect("Just set above"),
            ) {
                return Err(anyhow::anyhow!(
                    "At least one of `include_mapped` or `include_unmapped` must be true when using a BAM/SAM file."
                ));
            }
        }
        Ok(())
    }
    fn store_progress_output(&mut self, progress: &crate::transformations::reports::Progress) {
        self.progress_output = Some(progress.clone());
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut filter: ApproxOrExactFilter = if self.false_positive_rate == 0.0 {
            ApproxOrExactFilter::Exact(HashSet::new())
        } else {
            let seed = self
                .seed
                .expect("seed should be validated to exist when false_positive_rate > 0.0");
            ApproxOrExactFilter::Approximate(Box::new(reproducible_cuckoofilter(
                seed,
                initial_filter_elements(
                    &self.filename,
                    self.include_mapped.expect("Verified in validate_segments"),
                    self.include_unmapped
                        .expect("Verified in validate_segments"),
                ),
                self.false_positive_rate,
            )))
        };
        // read them all.
        if let Some(pg) = self.progress_output.as_mut() {
            pg.output(&format!(
                "Reading all read sequences from {}",
                self.filename
            ));
        }
        let count: Cell<usize> = Cell::new(0);

        crate::io::apply_to_read_sequences(
            &self.filename,
            &mut |read_seq| {
                if !filter.contains(&FragmentEntry(&[read_seq])) {
                    filter.insert(&FragmentEntry(&[read_seq]));
                }
                count.set(count.get() + 1);
            },
            self.include_mapped.expect("Verified in validate_segments"),
            self.include_unmapped
                .expect("Verified in validate_segments"),
        )?;
        if count.get() == 0 {
            bail!(
                "No read names were read from {}. Check that the file is not empty and (for BAM) that the include_mapped/include_unmapped options are set correctly.",
                self.filename
            );
        }
        if let Some(pg) = self.progress_output.as_mut() {
            pg.output(&format!(
                "Finished reading all ({}) read sequences from {}",
                count.get(),
                self.filename
            ));
        }

        self.filter = Some(Arc::new(filter));
        Ok(None)
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_bool_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read, _ignored_demultiplex_tag| {
                let filter = self
                    .filter
                    .as_ref()
                    .expect("filter must be set during initialization");
                let query = read.seq();
                filter.contains(&FragmentEntry(&[query]))
            },
        );

        Ok((block, true))
    }
}
