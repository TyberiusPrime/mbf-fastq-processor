use super::super::{FinalizeReportResult, InputInfo, Step, Transformation};
use crate::config::TargetPlusAll;
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use anyhow::Result;
use serde_json::{Map, Value};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct _ReportCountOligos {
    pub report_no: usize,
    pub oligos: Vec<String>,
    pub counts: Vec<Vec<usize>>,
    pub target: TargetPlusAll,
}

impl _ReportCountOligos {
    pub fn new(report_no: usize, oligos: &[String], target: TargetPlusAll) -> Self {
        let oligos = oligos.to_vec();
        Self {
            report_no,
            oligos,
            counts: Vec::new(),
            target,
        }
    }
}

impl Step for Box<_ReportCountOligos> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }
    fn validate(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        for _ in 0..=(demultiplex_info.max_tag()) {
            self.counts.push(vec![0; self.oligos.len()]);
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let mut blocks = Vec::new();
        match self.target {
            TargetPlusAll::Read1 => blocks.push(&block.read1),
            TargetPlusAll::Read2 => {
                if let Some(read2) = block.read2.as_ref() {
                    blocks.push(read2);
                }
            }
            TargetPlusAll::Index1 => {
                if let Some(index1) = block.index1.as_ref() {
                    blocks.push(index1);
                }
            }
            TargetPlusAll::Index2 => {
                if let Some(index2) = block.index2.as_ref() {
                    blocks.push(index2);
                }
            }
            TargetPlusAll::All => {
                blocks.push(&block.read1);
                if let Some(read2) = block.read2.as_ref() {
                    blocks.push(read2);
                }
                if let Some(index1) = block.index1.as_ref() {
                    blocks.push(index1);
                }
                if let Some(index2) = block.index2.as_ref() {
                    blocks.push(index2);
                }
            }
        }
        for read_iter in blocks {
            let mut iter = read_iter.get_pseudo_iter_including_tag(&block.output_tags);
            while let Some((read, demultiplex_tag)) = iter.pseudo_next() {
                let seq = read.seq();
                for (ii, oligo) in self.oligos.iter().enumerate() {
                    //todo: faster search algorithm...
                    if seq.windows(oligo.len()).any(|w| w == oligo.as_bytes()) {
                        self.counts[demultiplex_tag as usize][ii] += 1;
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
        let mut contents = Map::new();
        //needs updating for demultiplex
        match demultiplex_info {
            Demultiplexed::No => {
                for (ii, oligo) in self.oligos.iter().enumerate() {
                    contents.insert(oligo.clone(), self.counts[0][ii].into());
                }
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    let mut local = Map::new();
                    for (ii, oligo) in self.oligos.iter().enumerate() {
                        local.insert(oligo.clone(), self.counts[tag as usize][ii].into());
                    }
                    contents.insert(barcode.to_string(), local.into());
                }
            }
        }
        let mut final_contents = Map::new();
        final_contents.insert("count_oligos".to_string(), Value::Object(contents));

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: Value::Object(final_contents),
        }))
    }
}
