use std::path::Path;

use crate::transformations::prelude::*;

use super::super::FinalizeReportResult;
use crate::config::SegmentIndexOrAll;
use serde_json::{Map, Value};

#[derive(Debug, Clone)]
pub struct _ReportCountOligos {
    pub report_no: usize,
    pub oligos: Vec<String>,
    pub counts: DemultiplexedData<Vec<usize>>,
    pub segment_index: SegmentIndexOrAll,
}

impl _ReportCountOligos {
    pub fn new(report_no: usize, oligos: &[String], segment_index: SegmentIndexOrAll) -> Self {
        let oligos = oligos.to_vec();
        Self {
            report_no,
            oligos,
            counts: DemultiplexedData::default(),
            segment_index,
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

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        for valid_tag in demultiplex_info.iter_tags() {
            self.counts.insert(valid_tag, vec![0; self.oligos.len()]);
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut blocks = Vec::new();
        match &self.segment_index {
            SegmentIndexOrAll::Indexed(idx) => {
                blocks.push(&block.segments[*idx]);
            }
            SegmentIndexOrAll::All => {
                for segment in &block.segments {
                    blocks.push(segment);
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
                        self.counts.get_mut(&demultiplex_tag).expect("demultiplex tag must exist in counts")[ii] += 1;
                    }
                }
            }
        }
        Ok((block, true))
    }
    fn finalize(
        &mut self,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        let mut contents = Map::new();
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                for (ii, oligo) in self.oligos.iter().enumerate() {
                    contents.insert(oligo.clone(), self.counts.get(&0).expect("default tag 0 must exist in counts")[ii].into());
                }
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = Map::new();
                        for (ii, oligo) in self.oligos.iter().enumerate() {
                            local.insert(oligo.clone(), self.counts.get(tag).expect("tag must exist in counts")[ii].into());
                        }
                        contents.insert(name.to_string(), local.into());
                    }
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
