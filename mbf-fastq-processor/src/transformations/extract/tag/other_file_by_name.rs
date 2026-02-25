#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use std::cell::Cell;
use std::collections::HashSet;

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::transformations::read_name_canonical_prefix;
use crate::transformations::tag::initial_filter_elements;

/// Tag whether reads are in another file (by name)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct OtherFileByName {
    pub filename: String,

    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndex,

    pub out_label: String,
    pub seed: Option<u64>,
    pub false_positive_rate: f64,

    pub include_mapped: bool,
    pub include_unmapped: bool,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub fastq_readname_end_char: Option<u8>,
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub reference_readname_end_char: Option<u8>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl VerifyIn<PartialConfig> for PartialOtherFileByName {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        if let Some(filename) = self.filename.as_ref() {
            if Path::new(filename).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("bam") || ext.eq_ignore_ascii_case("sam")
            }) {
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
                let their_char: BString = BString::new(vec![info.comment_separator]);
                let our_char: BString = self
                    .fastq_readname_end_char
                    .map_or(b"/".into(), |x| BString::new(vec![x]));
                if their_char != our_char {
                    return Err(anyhow::anyhow!(
                        "OtherFileByName is configured to trim read names at character '{our_char}' (by option fastq_readname_end_char), but an upstream StoreTagInComment step is inserting comments that start with character '{their_char}' (option comment_separator). These must match.",
                    ));
                }
            }
        }

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
            self.include_mapped,
            self.include_unmapped,
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
            self.segment,
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
