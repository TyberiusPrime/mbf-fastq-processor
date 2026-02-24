#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use std::cell::RefCell;

use super::super::extract_bool_tags_plus_all;
use super::ApproxOrExactFilter;
use crate::dna::TagValue;
use crate::transformations::extract::extract_bool_tags_from_tag;
use crate::transformations::{read_name_canonical_prefix, tag::calculate_filter_capacity};


#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
/// Tag duplicate reads
pub struct Duplicates {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    source: ResolvedSourceAll,

    pub out_label: String,
    pub false_positive_rate: f64,

    pub seed: Option<u64>,

    pub initial_filter_capacity: Option<usize>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub filters: Arc<Mutex<DemultiplexedData<ApproxOrExactFilter>>>,
}

impl VerifyIn<PartialConfig> for PartialDuplicates {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        Ok(())
    }
}

impl Step for Duplicates {
    #[mutants::skip] // technically unecessary, since we have our own arc. But no point in blocking
    // multiple step-threads
    fn needs_serial(&self) -> bool {
        true
    }

    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[crate::transformations::Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate seed requirement based on false_positive_rate
        crate::transformations::tag::validate_seed(self.seed, self.false_positive_rate)
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Bool,
        ))
    }

    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[crate::transformations::TagValueType])>> {
        self.source.get_tags()
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_override: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Filters are initialized in apply() on first block for dynamic sizing
        Ok(None)
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
        block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize filters on first block using dynamic sizing
        if block_no == 1 {
            let seed = {
                if self.false_positive_rate > 0.0 {
                    self.seed
                        .expect("seed should be validated to exist when false_positive_rate > 0.0")
                } else {
                    42 // ignored anyway
                }
            };

            let capacity = calculate_filter_capacity(
                self.initial_filter_capacity,
                input_info,
                demultiplex_info.len(),
            );
            //dbg!(capacity);

            let mut filters = self.filters.lock().expect("Should have been provided by VerifyIn");
            for tag in demultiplex_info.iter_tags() {
                filters.insert(
                    tag,
                    ApproxOrExactFilter::new(self.false_positive_rate, capacity, seed),
                );
            }
        }

        match &self.source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                let filters = RefCell::new(
                    self.filters
                        .lock()
                        .expect("Failed to aquire filter lock") ,
                );
                extract_bool_tags_plus_all(
                    &mut block,
                    *segment_index_or_all,
                    &self.out_label,
                    |read, demultiplex_tag| {
                        filters
                            .borrow_mut()
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex_tag must exist in filters")
                            .containsert(&FragmentEntry(&[read.seq()]))
                    },
                    |reads, demultiplex_tag| {
                        // Virtually combine sequences for filter check
                        let inner: Vec<_> =
                            reads.iter().map(crate::io::WrappedFastQRead::seq).collect();
                        let entry = FragmentEntry(&inner);
                        filters
                            .borrow_mut()
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex_tag must exist in filters")
                            .containsert(&entry)
                    },
                );
            }
            ResolvedSourceAll::Tag(tag_name) => {
                let mut filters = self
                    .filters
                    .lock()
                    .expect("Failed to aquire filter lock")
                    ;
                extract_bool_tags_from_tag(
                    &mut block,
                    &self.out_label,
                    tag_name,
                    |tag_value, demultiplex_tag| {
                        if let Some(value) = Self::tag_value_to_bytes(tag_value) {
                            filters
                                .get_mut(&demultiplex_tag)
                                .expect("demultiplex_tag must exist in filters")
                                .containsert(&FragmentEntry(&[value.as_slice()]))
                        } else {
                            false
                        }
                    },
                );
            }
            ResolvedSourceAll::Name {
                segment_index_or_all,
                split_character,
            } => {
                let filters = RefCell::new(
                    self.filters
                        .lock()
                        .expect("Failed to aquire filter lock")
                );

                //todo: write a test case sthat tags duplicates in demultiplexd mode
                extract_bool_tags_plus_all(
                    &mut block,
                    *segment_index_or_all,
                    &self.out_label,
                    |read, demultiplex_tag| {
                        let name = read.name();
                        let canonical = read_name_canonical_prefix(name, Some(*split_character));
                        let owned = canonical.to_vec();
                        filters
                            .borrow_mut()
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex_tag must exist in filters")
                            .containsert(&FragmentEntry(&[owned.as_slice()]))
                    },
                    |reads, demultiplex_tag| {
                        // Virtually combine sequences for filter check
                        let inner: Vec<_> = reads
                            .iter()
                            .map(crate::io::WrappedFastQRead::name)
                            .collect();
                        let entry = FragmentEntry(&inner);
                        filters
                            .borrow_mut()
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex_tag must exist in filters")
                            .containsert(&entry)
                    },
                );
            }
        }
        Ok((block, true))
    }
}

impl Duplicates {
    fn tag_value_to_bytes(value: &TagValue) -> Option<Vec<u8>> {
        match value {
            TagValue::Location(hits) => Some(hits.joined_sequence(Some(&[0xff]))),
            TagValue::String(value) => Some(value.to_vec()),
            TagValue::Missing => None,
            _ => {
                unreachable!("Value was {:?}", value)
            }
        }
    }
}
