use bstr::ByteSlice;
use std::cell::RefCell;

use super::super::extract_bool_tags_plus_all;
use super::ApproxOrExactFilter;
use crate::transformations::extract::extract_bool_tags_from_tag;
use crate::transformations::prelude::*;
use crate::transformations::{read_name_canonical_prefix, tag::calculate_filter_capacity};
use fastqrab_io::io::WrappedFastQRead;

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
/// Tag duplicate reads
pub struct Duplicates {
    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    source: ResolvedSourceAll,

    pub out_label: TagLabel,
    pub false_positive_rate: f64,

    pub seed: Option<u64>,

    pub initial_filter_capacity: Option<usize>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    pub filters: Arc<Mutex<DemultiplexedData<ApproxOrExactFilter>>>,
}

impl VerifyIn<PartialConfig> for PartialDuplicates {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        crate::transformations::tag::validate_seed(&mut self.seed, &mut self.false_positive_rate);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialDuplicates> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = vec![];
            used_tags.extend(inner.source.to_used_tags());
            Some(TagUsageInfo {
                used_tags,
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Bool),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Duplicates {
    #[mutants::skip] // technically unecessary, since we have our own arc. But no point in blocking
    // multiple step-threads
    fn needs_serial(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Filters are initialized in apply() on first block for dynamic sizing
        Ok(None)
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize filters on first block using dynamic sizing
        if block.block_no() == 1 {
            let seed = self.seed.unwrap_or(42);

            let capacity = calculate_filter_capacity(
                self.initial_filter_capacity,
                input_info,
                demultiplex_info.len(),
            );
            //dbg!(capacity);

            let mut filters = self
                .filters
                .lock()
                .expect("Should have been provided by VerifyIn");
            for tag in demultiplex_info.iter_tags() {
                filters.insert(
                    tag,
                    ApproxOrExactFilter::new(self.false_positive_rate, capacity, seed),
                );
            }
        }

        match &self.source {
            ResolvedSourceAll::Segment(segment_index_or_all) => {
                let filters =
                //safe, we're single threaded in this
                    RefCell::new(self.filters.lock().expect("Failed to aquire filter lock"));
                extract_bool_tags_plus_all(
                    &mut block,
                    *segment_index_or_all,
                    &self.out_label,
                    |read, demultiplex_tag| {
                        filters
                            .borrow_mut()
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex_tag must exist in filters")
                            .containsert(&FragmentEntry(&[read.seq]))
                    },
                    |reads, demultiplex_tag| {
                        // Virtually combine sequences for filter check
                        let inner: Vec<_> = reads.iter().map(|read| read.seq.as_bytes()).collect();
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
                let mut filters = self.filters.lock().expect("Failed to aquire filter lock");
                extract_bool_tags_from_tag(
                    &mut block,
                    &self.out_label,
                    tag_name,
                    |query, demultiplex_tag| {
                        if let Some(query) = query {
                            filters
                                .get_mut(&demultiplex_tag)
                                .expect("demultiplex_tag must exist in filters")
                                .containsert(&FragmentEntry(&[&query]))
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
                let filters =
                    RefCell::new(self.filters.lock().expect("Failed to aquire filter lock"));

                //todo: write a test case sthat tags duplicates in demultiplexd mode
                extract_bool_tags_plus_all(
                    &mut block,
                    *segment_index_or_all,
                    &self.out_label,
                    |read, demultiplex_tag| {
                        let name = read.name;
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
                        let inner: Vec<_> = reads.iter().map(|read| read.name.as_bytes()).collect();
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
