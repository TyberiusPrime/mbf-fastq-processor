#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::transformations::extract::extract_bool_tags_from_tag;
use crate::transformations::{prelude::*, read_name_canonical_prefix};

use std::cell::Cell;
use std::collections::HashSet;

use super::super::extract_bool_tags;
use super::ApproxOrExactFilter;
use crate::transformations::tag::initial_filter_elements;

/// Tag whether reads are in another file (by sequence)
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct OtherFile {
    pub filename: String,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String), alias = "segment")]
    source: ResolvedSourceNoAll,

    #[tpd(
        with = "tpd_adapt_u8_from_byte_or_char",
        alias = "other_read_name_end_char"
    )]
    pub other_readname_end_character: Option<u8>,

    pub out_label: String,

    pub seed: Option<u64>,
    pub false_positive_rate: f64,

    pub include_mapped: bool,
    pub include_unmapped: bool,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub filter: Option<Arc<ApproxOrExactFilter>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub progress_output: Option<crate::transformations::reports::Progress>,
}

impl VerifyIn<PartialConfig> for PartialOtherFile {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        //todo: refactor with OtherFileByName to avoid code duplication.
        if let Some(filename) = self.filename.as_ref()
            && Path::new(filename)
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("bam") || x.eq_ignore_ascii_case("sam"))
        {
            if self.include_unmapped.is_missing() {
                self.include_unmapped.or(false); // just so it's always set, but still report the
                // error.
                return Err(ValidationFailure::new(
                    "Missing include_unmapped",
                    Some("When using a BAM file, you must specify `include_unmapped` = true|false"),
                ));
            }

            if self.include_mapped.is_missing() {
                self.include_mapped.or(false); // just so it's always set, but still report the
                // error.
                return Err(ValidationFailure::new(
                    "Missing include_mapped",
                    Some("When using a BAM file, you must specify `include_mapped` = true|false"),
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
        self.source.verify(|v| {
            if let Some(output_tag) = self.out_label.as_ref()
                && let MustAdapt::PostVerify(ResolvedSourceNoAll::Tag(input_tag)) = v
            {
                if output_tag == input_tag {
                    return Err(ValidationFailure::new(
                        "Source cannot be the same as output tag",
                        Some("The source (segment or tag) cannot be the same as the output tag"),
                    ));
                }
            }
            Ok(())
        });
        Ok(())
    }
}

impl Step for OtherFile {
    #[allow(clippy::case_sensitive_file_extension_comparisons)]
    fn validate_others(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        all_transforms: &[Transformation],
        this_transforms_index: usize,
    ) -> Result<()> {
        if let ResolvedSourceNoAll::Name { .. } = &self.source {
            //if there's a StoreTagInComment before us
            //and our fastq_readname_end_char is != their comment_insert_char
            //bail
            for trafo in all_transforms[..this_transforms_index].iter().rev() {
                if let Transformation::StoreTagInComment(info) = trafo {
                    let their_char: BString = BString::new(vec![info.comment_separator]);
                    let our_char: BString =
                        BString::new(vec![input_def.options.read_comment_character]);
                    if their_char != our_char {
                        return Err(anyhow::anyhow!(
                            "OtherFile using names is configured to trim read names at character '{our_char}'
(by `input.options.read_comment_character`), 
but an upstream StoreTagInComment step is inserting comments that start with character '{their_char}'
(option comment_separator).
These must match.",
                        ));
                    }
                }
            }
        }
        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn store_progress_output(&mut self, progress: &crate::transformations::reports::Progress) {
        self.progress_output = Some(progress.clone());
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        if let ResolvedSourceNoAll::Tag(tag) = &self.source {
            Some(vec![(
                tag.clone(),
                &[TagValueType::String, TagValueType::Location],
            )])
        } else {
            None
        }
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
            pg.output(&format!("Reading all reads from {}", self.filename));
        }
        let count: Cell<usize> = Cell::new(0);

        match self.source {
            ResolvedSourceNoAll::Segment(_) | ResolvedSourceNoAll::Tag(_) => {
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
            }
            ResolvedSourceNoAll::Name { .. } => {
                crate::io::apply_to_read_names(
                    &self.filename,
                    &mut |read_name| {
                        let trimmed = read_name_canonical_prefix(
                            read_name,
                            self.other_readname_end_character,
                        );

                        if !filter.contains(&FragmentEntry(&[trimmed])) {
                            filter.insert(&FragmentEntry(&[trimmed]));
                        }
                        count.set(count.get() + 1);
                    },
                    self.include_mapped,
                    self.include_unmapped,
                )?;
            }
        }

        if count.get() == 0 {
            bail!(
                "No reads were read from {}. Check that the file is not empty and (for BAM) that the include_mapped/include_unmapped options are set correctly.",
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
        match &self.source {
            ResolvedSourceNoAll::Segment(segment_index) => {
                extract_bool_tags(
                    &mut block,
                    *segment_index,
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
            }
            ResolvedSourceNoAll::Tag(tag) => {
                extract_bool_tags_from_tag(
                    &mut block,
                    tag,
                    &self.out_label,
                    |tag_value, _ignored_demultiplex_tag| {
                        let filter = self
                            .filter
                            .as_ref()
                            .expect("filter must be set during initialization");
                        let query = tag_value
                            .as_str(b"")
                            .expect("Input tag value must be a string");
                        filter.contains(&FragmentEntry(&[&query]))
                    },
                );
            }
            ResolvedSourceNoAll::Name {
                segment_index,
                split_character,
            } => {
                extract_bool_tags(
                    &mut block,
                    *segment_index,
                    &self.out_label,
                    |read, _ignored_demultiplex_tag| {
                        let query = read_name_canonical_prefix(read.name(), Some(*split_character));

                        self.filter
                            .as_ref()
                            .expect("filter must be set during initialization")
                            .contains(&FragmentEntry(&[query]))
                    },
                );
            }
        }

        Ok((block, true))
    }
}
