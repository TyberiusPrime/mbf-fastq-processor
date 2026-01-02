use crate::transformations::prelude::*;

use super::super::{
    FinalizeReportResult, FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter,
    reproducible_cuckoofilter,
};
use crate::{io::WrappedFastQRead, transformations::tag::calculate_filter_capacity};
use std::path::Path;

#[derive(Default, Debug, Clone)]
pub struct DuplicateFragmentCountData {
    duplicate_count: usize,
    duplication_filter: Option<OurCuckCooFilter<FragmentEntryForCuckooFilter>>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for DuplicateFragmentCountData {
    fn into(self) -> serde_json::Value {
        self.duplicate_count.into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportDuplicateFragmentCount {
    pub report_no: usize,
    //that is per read1/read2...
    pub data: Arc<Mutex<DemultiplexedData<DuplicateFragmentCountData>>>,
    pub debug_reproducibility: bool,
    pub initial_filter_capacity: Arc<Mutex<Option<usize>>>,
    pub actual_filter_capacity: Option<usize>,
}

impl Step for Box<_ReportDuplicateFragmentCount> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    // fn needs_serial(&self) -> bool {
    //     true
    // }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Initialize data structures but not the filters (those are initialized in apply)
        let mut data_lock = self.data.lock().expect("lock poisened");
        for valid_tag in demultiplex_info.iter_tags() {
            data_lock.insert(
                valid_tag,
                DuplicateFragmentCountData {
                    duplicate_count: 0,
                    duplication_filter: None, // Initialized in apply() on first block
                },
            );
        }
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize filters on first block using dynamic sizing
        let mut data_lock = self.data.lock().expect("lock poisened");
        if block_no == 1 {
            let false_positive_probability = if self.debug_reproducibility {
                0.1
            } else {
                0.01
            };
            let capacity = calculate_filter_capacity(
                *self.initial_filter_capacity.lock().expect("lock poisened"),
                input_info,
                demultiplex_info.len(),
            );

            for tag in demultiplex_info.iter_tags() {
                let data = data_lock.get_mut(&tag).expect("Tag checked during init");
                data.duplication_filter = Some(reproducible_cuckoofilter(
                    42,
                    capacity,
                    false_positive_probability,
                ));
            }
        }

        {
            let mut block_iter = block.get_pseudo_iter();
            let pos = 0;
            let demultiplex_tags = block.output_tags.as_ref();
            while let Some(molecule) = block_iter.pseudo_next() {
                let inner: Vec<_> = molecule
                    .segments
                    .iter()
                    .map(WrappedFastQRead::seq)
                    .collect();
                let seq = FragmentEntry(&inner);
                // passing in this complex/reference type into the cuckoo_filter
                // is a nightmare.
                let tag = demultiplex_tags.map_or(0, |x| x[pos]);
                let target = data_lock
                    .get_mut(&tag)
                    .expect("demultiplextag must exist in data");
                //todo: use once this is released in scalable_cuckoofilter
                // if target
                //     .duplication_filter
                //     .as_mut()
                //     .expect("duplication_filter must be set during initialization")
                //     .insert_if_not_contained(&seq)
                // {
                //     target.duplicate_count += 1;
                // }

                if target
                    .duplication_filter
                    .as_ref()
                    .expect("duplication_filter must be set during initialization")
                    .contains(&seq)
                {
                    target.duplicate_count += 1;
                } else {
                    target
                        .duplication_filter
                        .as_mut()
                        .expect("duplication_filter must be set during initialization")
                        .insert(&seq);
                }
            }
        }
        Ok((block, true))
    }

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        let data_lock = self.data.lock().expect("lock poisened");

        // Add filter capacity information if available
        if let Some(capacity) = *self.initial_filter_capacity.lock().expect("lock poisened") {
            contents.insert(
                "initial_filter_capacity".to_string(),
                serde_json::Value::Number(capacity.into()),
            );
        }
        let actual_filter_capacity = data_lock
            .values()
            .next()
            .and_then(|data| data.duplication_filter.as_ref())
            .map(scalable_cuckoo_filter::ScalableCuckooFilter::capacity)
            .expect("Could not retrieve filter capacity? Bug");
        contents.insert(
            "actual_filter_capacity".to_string(),
            serde_json::Value::Number(actual_filter_capacity.into()),
        );

        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                contents.insert(
                    "fragment_duplicate_count".to_string(),
                    data_lock
                        .get(&0)
                        .expect("tag 0 must exist in data")
                        .duplicate_count
                        .into(),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        local.insert(
                            "fragment_duplicate_count".to_string(),
                            data_lock
                                .get(tag)
                                .expect("tag must exist in data")
                                .duplicate_count
                                .into(),
                        );
                        contents.insert(name.to_string(), local.into());
                    }
                }
            }
        }

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}
