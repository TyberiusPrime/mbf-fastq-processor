use super::super::{FinalizeReportResult, InputInfo, Step};
use super::common::PerReadReportData;
use crate::{
    demultiplex::{DemultiplexInfo, Demultiplexed},
    io,
};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct _ReportLengthDistribution {
    pub report_no: usize,
    pub data: Vec<PerReadReportData<Vec<usize>>>,
}

impl _ReportLengthDistribution {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Vec::default(),
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
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        for _ in 0..=(demultiplex_info.max_tag()) {
            self.data.push(PerReadReportData::new(input_info));
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
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
            let output = &mut self.data[tag as usize];
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
                self.data[0].store("length_distribution", &mut contents);
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    let mut local = serde_json::Map::new();
                    self.data[tag as usize].store("length_distribution", &mut local);
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
