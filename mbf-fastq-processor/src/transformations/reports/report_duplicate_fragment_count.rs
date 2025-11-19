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
    pub data: DemultiplexedData<DuplicateFragmentCountData>,
    pub debug_reproducibility: bool,
    pub initial_filter_capacity: Option<usize>,
    pub actual_filter_capacity: Option<usize>,
}

impl Step for Box<_ReportDuplicateFragmentCount> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

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
        for valid_tag in demultiplex_info.iter_tags() {
            self.data.insert(
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
        &mut self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        // Initialize filters on first block using dynamic sizing
        if block_no == 1 {
            let false_positive_probability = if self.debug_reproducibility {
                0.1
            } else {
                0.01
            };
            let capacity = calculate_filter_capacity(
                self.initial_filter_capacity,
                input_info,
                demultiplex_info.len(),
            );

            self.actual_filter_capacity = Some(capacity);

            for tag in demultiplex_info.iter_tags() {
                let data = self.data.get_mut(&tag).unwrap();
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
            while let Some(molecule) = block_iter.pseudo_next() {
                let inner: Vec<_> = molecule
                    .segments
                    .iter()
                    .map(WrappedFastQRead::seq)
                    .collect();
                let seq = FragmentEntry(&inner);
                // passing in this complex/reference type into the cuckoo_filter
                // is a nightmare.
                let tag = block.output_tags.as_ref().map_or(0, |x| x[pos]);
                let target = self.data.get_mut(&tag).expect("tag must exist in data");
                if target.duplication_filter.as_ref().expect("duplication_filter must be set during initialization").contains(&seq) {
                    target.duplicate_count += 1;
                } else {
                    target.duplication_filter.as_mut().expect("duplication_filter must be set during initialization").insert(&seq);
                }
            }
        }
        Ok((block, true))
    }

    fn finalize(
        &mut self,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();

        // Add filter capacity information if available
        if let Some(capacity) = self.initial_filter_capacity {
            contents.insert(
                "initial_filter_capacity".to_string(),
                serde_json::Value::Number(capacity.into()),
            );
        }
        if let Some(capacity) = self.actual_filter_capacity {
            contents.insert(
                "actual_filter_capacity".to_string(),
                serde_json::Value::Number(capacity.into()),
            );
        }

        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                contents.insert(
                    "fragment_duplicate_count".to_string(),
                    self.data.get(&0).expect("tag 0 must exist in data").duplicate_count.into(),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        local.insert(
                            "fragment_duplicate_count".to_string(),
                            self.data.get(tag).expect("tag must exist in data").duplicate_count.into(),
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
