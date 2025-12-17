#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use std::cell::Cell;
use std::sync::Arc;
use std::{collections::HashSet, path::Path};

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::config::deser::single_u8_from_string;
use crate::transformations::read_name_canonical_prefix;
use crate::transformations::tag::initial_filter_elements;
use crate::transformations::{FragmentEntry, InputInfo, reproducible_cuckoofilter};
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Validate, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtherFileByName {
    pub filename: String,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,
    pub seed: Option<u64>,
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub false_positive_rate: f64,

    #[serde(default)]
    pub include_mapped: Option<bool>,
    #[serde(default)]
    pub include_unmapped: Option<bool>,

    #[serde(default, deserialize_with = "single_u8_from_string")]
    pub fastq_readname_end_char: Option<u8>,

    #[serde(default, deserialize_with = "single_u8_from_string")]
    pub reference_readname_end_char: Option<u8>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

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
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        //if there's a StoreTagInComment before us
        //and our fastq_readname_end_char is != their comment_insert_char
        //bail
        for trafo in all_transforms[..this_transforms_index].iter().rev() {
            if let crate::Transformation::StoreTagInComment(info) = trafo {
                let their_char: Option<BString> = Some(BString::new(vec![info.comment_separator]));
                let our_char: Option<BString> =
                    self.fastq_readname_end_char.map(|x| BString::new(vec![x]));
                if their_char != our_char {
                    return Err(anyhow::anyhow!(
                        "OtherFileByName is configured to trim read names at character '{our_char:?}' (by option fastq_readname_end_char), but an upstream StoreTagInComment step is inserting comments that start with character '{their_char:?}' (option comment_separator). These must match.",
                    ));
                }
            }
        }

        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn store_progress_output(&mut self, progress: &crate::transformations::reports::Progress) {
        self.progress_output = Some(progress.clone());
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

        Ok(())
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
            pg.output(&format!("Reading all read names from {}", self.filename));
        }

        let counter = Cell::new(0);
        crate::io::apply_to_read_names(
            &self.filename,
            &mut |read_name| {
                let trimmed =
                    read_name_canonical_prefix(read_name, self.reference_readname_end_char);

                if !filter.contains(&FragmentEntry(&[trimmed])) {
                    filter.insert(&FragmentEntry(&[trimmed]));
                }
                counter.set(counter.get() + 1);
            },
            self.include_mapped.expect("Verified in validate_segments"),
            self.include_unmapped
                .expect("Verified in validate_segments"),
        )?;
        if let Some(pg) = self.progress_output.as_mut() {
            pg.output(&format!(
                "Finished reading all ({}) read names from {}",
                counter.get(),
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
        let count: Cell<usize> = Cell::new(0);
        extract_bool_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read, _ignored_demultiplex_tag| {
                count.set(count.get() + 1);
                let query = read_name_canonical_prefix(read.name(), self.fastq_readname_end_char);

                self.filter
                    .as_ref()
                    .expect("filter must be set during initialization")
                    .contains(&FragmentEntry(&[query]))
            },
        );

        Ok((block, true))
    }
}
