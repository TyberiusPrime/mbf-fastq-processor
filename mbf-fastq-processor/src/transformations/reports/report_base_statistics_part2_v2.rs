use crate::transformations::prelude::*;

use super::common::{BASE_TO_INDEX, PerReadReportData, PositionCount};
use crate::io;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct BaseStatisticsPart2V2 {
    per_position_counts: Vec<PositionCount>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for BaseStatisticsPart2V2 {
    fn into(self) -> serde_json::Value {
        let c = self
            .per_position_counts
            .iter()
            .map(|x| x.0[1])
            .collect::<Vec<_>>();
        let g = self
            .per_position_counts
            .iter()
            .map(|x| x.0[2])
            .collect::<Vec<_>>();
        let gc_bases: usize = c.iter().sum::<usize>() + g.iter().sum::<usize>();
        let position_counts = json!({
            "a": self.per_position_counts.iter().map(|x| x.0[0]).collect::<Vec<_>>(),
            "c": c,
            "g": g,
            "t": self.per_position_counts.iter().map(|x| x.0[3]).collect::<Vec<_>>(),
            "n": self.per_position_counts.iter().map(|x| x.0[4]).collect::<Vec<_>>(),
        });

        json!({
            "gc_bases": gc_bases,
            "per_position_counts": position_counts
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportBaseStatisticsPart2V2 {
    pub report_no: usize,
    pub data: DemultiplexedData<PerReadReportData<BaseStatisticsPart2V2>>,
}

impl _ReportBaseStatisticsPart2V2 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportBaseStatisticsPart2V2> {
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
        fn update_from_read(target: &mut BaseStatisticsPart2V2, read: &io::WrappedFastQRead) {
            let read_len = read.len();
            if target.per_position_counts.len() <= read_len {
                target
                    .per_position_counts
                    .resize(read_len, PositionCount([0; 5]));
            }
            let seq: &[u8] = read.seq();

            // Optimized: use unsafe to eliminate bounds checking
            // Safety: We just resized to ensure read_len capacity, and we only iterate up to read_len
            // BASE_TO_INDEX always returns 0-4, which is within bounds of the [0; 5] array
            for ii in 0..read_len {
                unsafe {
                    let base = *seq.get_unchecked(ii);
                    let idx = *BASE_TO_INDEX.get_unchecked(base as usize);
                    let counts = target.per_position_counts.get_unchecked_mut(ii);
                    *counts.0.get_unchecked_mut(idx as usize) += 1;
                }
            }
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = self.data.get_mut(&tag).unwrap();
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
                    .store("base_statistics", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        self.data
                            .get(tag)
                            .unwrap()
                            .store("base_statistics", &mut local);
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
