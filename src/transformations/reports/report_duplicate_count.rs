use super::super::{
    FinalizeReportResult, InputInfo, OurCuckCooFilter, Step, reproducible_cuckoofilter,
};
use super::common::PerReadReportData;
use crate::{
    demultiplex::{DemultiplexInfo, Demultiplexed},
    io,
};
use anyhow::Result;
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
    pub data_per_read: Vec<PerReadReportData<DuplicateCountData>>,
    pub debug_reproducibility: bool,
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
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        let (initial_capacity, false_positive_probability) = if self.debug_reproducibility {
            (100, 0.1)
        } else {
            (1_000_000, 0.01)
        };

        for _ in 0..=(demultiplex_info.max_tag()) {
            self.data_per_read.push(PerReadReportData {
                read1: Some(DuplicateCountData {
                    duplicate_count: 0,
                    duplication_filter: Some(reproducible_cuckoofilter(
                        42,
                        initial_capacity,
                        false_positive_probability,
                    )),
                }),
                read2: input_info.has_read2.then(|| DuplicateCountData {
                    duplicate_count: 0,
                    duplication_filter: Some(reproducible_cuckoofilter(
                        42,
                        initial_capacity,
                        false_positive_probability,
                    )),
                }),
                index1: input_info.has_index1.then(|| DuplicateCountData {
                    duplicate_count: 0,
                    duplication_filter: Some(reproducible_cuckoofilter(
                        42,
                        initial_capacity,
                        false_positive_probability,
                    )),
                }),
                index2: input_info.has_index2.then(|| DuplicateCountData {
                    duplicate_count: 0,
                    duplication_filter: Some(reproducible_cuckoofilter(
                        42,
                        initial_capacity,
                        false_positive_probability,
                    )),
                }),
            });
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
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
            let output = &mut self.data_per_read[tag as usize];
            for (storage, read_block) in [
                (&mut output.read1, Some(&block.read1)),
                (&mut output.read2, block.read2.as_ref()),
                (&mut output.index1, block.index1.as_ref()),
                (&mut output.index2, block.index2.as_ref()),
            ] {
                if read_block.is_some() {
                    let mut iter = match &block.output_tags {
                        Some(output_tags) => read_block
                            .as_ref()
                            .unwrap()
                            .get_pseudo_iter_filtered_to_tag(tag, output_tags),
                        None => read_block.as_ref().unwrap().get_pseudo_iter(),
                    };
                    while let Some(read) = iter.pseudo_next() {
                        update_from_read(storage.as_mut().unwrap(), &read);
                    }
                }
            }
        }
        (block, true)
    }

    fn finalize(
        &mut self,
        _output_prefix: &str,
        _output_directory: &Path,
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        //needs updating for demultiplex
        match demultiplex_info {
            Demultiplexed::No => {
                self.data_per_read[0].store("duplicate_count", &mut contents);
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    let mut local = serde_json::Map::new();
                    self.data_per_read[tag as usize].store("duplicate_count", &mut local);
                    contents.insert(barcode.to_string(), local.into());
                }
            }
        }

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}
