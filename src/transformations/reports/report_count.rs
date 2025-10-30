use super::super::{FinalizeReportResult, InputInfo, Step};
use crate::demultiplex::{Demultiplex, DemultiplexInfo, Demultiplexed};
use anyhow::Result;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct _ReportCount {
    pub report_no: usize,
    pub data: Vec<usize>,
}

impl _ReportCount {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Vec::new(),
        }
    }
}

impl Step for Box<_ReportCount> {
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
        demultiplex_info: &Demultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexInfo>> {
        //if there's a demultiplex step *before* this report,
        //
        for _ in 0..=(demultiplex_info.demultiplexed.max_tag()) {
            self.data.push(0);
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        match &demultiplex_info.demultiplexed {
            Demultiplexed::No => self.data[0] += block.len(),
            Demultiplexed::Yes(_) => {
                for tag in block.output_tags.as_ref().unwrap() {
                    self.data[*tag as usize] += 1;
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
        demultiplex_info: &Demultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        //needs updating for demultiplex
        match &demultiplex_info.demultiplexed {
            Demultiplexed::No => {
                contents.insert("molecule_count".to_string(), self.data[0].into());
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    contents.insert(
                        barcode.to_string(),
                        json!({
                            "molecule_count": self.data[tag as usize],
                        }),
                    );
                }
            }
        }

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}
