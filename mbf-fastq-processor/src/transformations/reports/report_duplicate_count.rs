use crate::transformations::prelude::*;

use super::super::{FinalizeReportResult, OurCuckCooFilter, reproducible_cuckoofilter};
use super::common::PerReadReportData;
use crate::{io, transformations::tag::calculate_filter_capacity};
use std::path::Path;

#[derive(Default, Debug, Clone)]
pub struct DuplicateCountData {
    duplicate_count: usize,
    duplication_filter: Option<OurCuckCooFilter<[u8]>>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for DuplicateCountData {
    fn into(self) -> serde_json::Value {
        self.duplicate_count.into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportDuplicateCount {
    pub report_no: usize,
    //that is per read1/read2...
    pub data_per_read: DemultiplexedData<PerReadReportData<DuplicateCountData>>,
    pub debug_reproducibility: bool,
    pub initial_filter_capacity: Option<usize>,
    #[serde(default)]
    #[serde(skip)]
    pub actual_filter_capacity: Option<usize>,
}

impl Step for Box<_ReportDuplicateCount> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Initialize data structures but not the filters (those are initialized in apply)
        for valid_tag in demultiplex_info.iter_tags() {
            let mut data_per_read = Vec::new();
            for segment_name in &input_info.segment_order {
                data_per_read.push((
                    segment_name.clone(),
                    DuplicateCountData {
                        duplicate_count: 0,
                        duplication_filter: None, // Initialized in apply() on first block
                    },
                ));
            }
            self.data_per_read.insert(
                valid_tag,
                PerReadReportData {
                    segments: data_per_read,
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
            let false_positive_probability = if self.debug_reproducibility { 0.1 } else { 0.01 };
            let capacity = calculate_filter_capacity(
                self.initial_filter_capacity,
                input_info,
                demultiplex_info.len(),
                self.debug_reproducibility,
            );

            self.actual_filter_capacity = Some(capacity);

            for tag in demultiplex_info.iter_tags() {
                let output = self.data_per_read.get_mut(&tag).unwrap();
                for (_segment_name, data) in &mut output.segments {
                    data.duplication_filter = Some(reproducible_cuckoofilter(
                        42,
                        capacity,
                        false_positive_probability,
                    ));
                }
            }
        }

        fn update_from_read(target: &mut DuplicateCountData, read: &io::WrappedFastQRead) {
            let seq = read.seq();
            if target.duplication_filter.as_ref().unwrap().contains(seq) {
                target.duplicate_count += 1;
            } else {
                target.duplication_filter.as_mut().unwrap().insert(seq);
            }
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = self.data_per_read.get_mut(&tag).unwrap();

            for (ii, read_block) in block.segments.iter().enumerate() {
                let storage = &mut output.segments[ii].1;
                let mut iter = match &block.output_tags {
                    Some(output_tags) => {
                        read_block.get_pseudo_iter_filtered_to_tag(tag, output_tags)
                    }
                    None => read_block.get_pseudo_iter(),
                };
                while let Some(read) = iter.pseudo_next() {
                    update_from_read(storage, &read);
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
        if let Some(capacity) = self.actual_filter_capacity {
            contents.insert(
                "filter_capacity".to_string(),
                serde_json::Value::Number(capacity.into()),
            );
        }

        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                self.data_per_read
                    .get(&0)
                    .unwrap()
                    .store("duplicate_count", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        self.data_per_read
                            .get(tag)
                            .unwrap()
                            .store("duplicate_count", &mut local);
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
