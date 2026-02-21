#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use std::cell::Cell;
use std::collections::HashSet;

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::transformations::tag::initial_filter_elements;

/// Tag whether reads are in another file (by sequence)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct OtherFileBySequence {
    pub filename: String,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    pub out_label: String,

    pub seed: Option<u64>,
    pub false_positive_rate: f64,

    pub include_mapped: bool,
    pub include_unmapped: bool,

    #[tpd(skip)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

    #[tpd(skip)]
    #[schemars(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl VerifyIn<PartialConfig> for PartialOtherFileBySequence {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        //todo: refactor with OtherFileByName to avoid code duplication.
        if let Some(filename) = self.filename.as_ref() {
            if (filename.ends_with(".bam") || filename.ends_with(".sam")) {
                if self.include_unmapped.is_missing() {
                    return Err(ValidationFailure::new(
                        "Missing include_unmapped",
                        Some(
                            "When using a BAM file, you must specify `include_unmapped` = true|false",
                        ),
                    ));
                }

                if self.include_mapped.is_missing() {
                    return Err(ValidationFailure::new(
                        "Missing include_mapped",
                        Some(
                            "When using a BAM file, you must specify `include_mapped` = true|false",
                        ),
                    ));
                }
                if !(self.include_mapped.value.unwrap_or(false)
                    || self.include_unmapped.value.unwrap_or(false))
                {
                    return Err(ValidationFailure::new(
                        "Invalid include_mapped/include_unmapped combination",
                        Some(
                            "At least one of `include_mapped` or `include_unmapped` must be true when using a BAM/SAM file.",
                        ),
                    ));
                }
            } else {
                self.include_mapped.or(false); // just so it's always set.
                self.include_unmapped.or(false);
            }
        }
        Ok(())
    }
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
                initial_filter_elements(&self.filename, self.include_mapped, self.include_unmapped),
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
            self.include_mapped,
            self.include_unmapped,
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
            self.segment,
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
