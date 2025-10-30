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
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexInfo>> {
        let (initial_capacity, false_positive_probability) = if self.debug_reproducibility {
            (100, 0.1)
        } else {
            (1_000_000, 0.01)
        };

        for _ in 0..=(demultiplex_info.max_tag()) {
            let mut data_per_read = Vec::new();
            for segment_name in &input_info.segment_order {
                data_per_read.push((
                    segment_name.clone(),
                    DuplicateCountData {
                        duplicate_count: 0,
                        duplication_filter: Some(reproducible_cuckoofilter(
                            42,
                            initial_capacity,
                            false_positive_probability,
                        )),
                    },
                ));
            }
            self.data_per_read.push(PerReadReportData {
                segments: data_per_read,
            });
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
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
        _input_info: &crate::transformations::InputInfo,
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
