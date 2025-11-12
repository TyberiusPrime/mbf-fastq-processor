use crate::transformations::prelude::*;

use super::super::FinalizeReportResult;
use super::common::PerReadReportData;
use crate::io;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct _ReportLengthDistribution {
    pub report_no: usize,
    pub data: DemultiplexedData<PerReadReportData<Vec<usize>>>,
}

impl _ReportLengthDistribution {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportLengthDistribution> {
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
        for valid_tag in demultiplex_info.iter_tags() {
            self.data
                .insert(valid_tag, PerReadReportData::new(input_info));
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn update_from_read(target: &mut Vec<usize>, read: &io::WrappedFastQRead) {
            let read_len = read.len();
            if target.len() <= read_len {
                //println!("Had to resize report buffer, {read_len}");
                target.resize(read_len + 1, 0);
            }
            target[read_len] += 1;
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = &mut self.data.get_mut(&tag).unwrap();
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
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                self.data
                    .get(&0)
                    .unwrap()
                    .store("length_distribution", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        self.data
                            .get(&tag)
                            .unwrap()
                            .store("length_distribution", &mut local);
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
