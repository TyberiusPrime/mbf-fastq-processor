#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use std::cell::Cell;
use std::{collections::HashSet};

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::config::deser::{single_u8_from_string, tpd_extract_u8_from_byte_or_char};
use crate::transformations::read_name_canonical_prefix;
use crate::transformations::tag::initial_filter_elements;

/// Tag whether reads are in another file (by name)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct OtherFileByName {
    pub filename: String,
    #[tpd_default]
    segment: Segment,

    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,
    pub seed: Option<u64>,
    pub false_positive_rate: f64,

    pub include_mapped: Option<bool>,
    pub include_unmapped: Option<bool>,

    #[tpd_adapt_in_verify]
    pub fastq_readname_end_char: Option<u8>,

    #[tpd_adapt_in_verify]
    pub reference_readname_end_char: Option<u8>,

    #[tpd_skip] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

    #[tpd_skip] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl VerifyFromToml for PartialOtherFileByName {
    fn verify(mut self, helper: &mut TomlHelper<'_>) -> Self
    where
        Self: Sized,
    {
        self.fastq_readname_end_char = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_fastq_readname_end_char(helper, false, false),
            self.tpd_get_fastq_readname_end_char(helper, false, false),
        ).into_optional();
        self.reference_readname_end_char = tpd_extract_u8_from_byte_or_char(
            self.tpd_get_reference_readname_end_char(helper, false, false),
            self.tpd_get_reference_readname_end_char(helper, false, false),
        ).into_optional();
        self
    }
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
            if let Transformation::StoreTagInComment(info) = trafo {
                let their_char: Option<BString> = Some(BString::new(vec![info.comment_separator]));
                let our_char: Option<BString> =
                    self.fastq_readname_end_char.map(|x| BString::new(vec![x]));
                if their_char != our_char {
                    return Err(anyhow::anyhow!(
                        "OtherFileByName is configured to trim read names at character '{}' (by option fastq_readname_end_char), but an upstream StoreTagInComment step is inserting comments that start with character '{}' (option comment_separator). These must match.",
                        our_char.unwrap_or(b"/".into()),
                        their_char.unwrap_or(b"/".into())
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
        if counter.get() == 0 {
            bail!(
                "No read names were read from {}. Check that the file is not empty and (for BAM) that the include_mapped/include_unmapped options are set correctly.",
                self.filename
            );
        }
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

    #[mutants::skip] // counter is explicitly not covered by tests.
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
