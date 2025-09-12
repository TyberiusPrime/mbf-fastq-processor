use super::super::{FinalizeReportResult, InputInfo, Step};
use super::common::{BASE_TO_INDEX, PerReadReportData, PositionCount};
use crate::{
    demultiplex::{DemultiplexInfo, Demultiplexed},
    io,
};
use anyhow::Result;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct BaseStatisticsPart2 {
    per_position_counts: Vec<PositionCount>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for BaseStatisticsPart2 {
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
pub struct _ReportBaseStatisticsPart2 {
    pub report_no: usize,
    pub data: Vec<PerReadReportData<BaseStatisticsPart2>>,
}

impl _ReportBaseStatisticsPart2 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Vec::default(),
        }
    }
}

impl Step for Box<_ReportBaseStatisticsPart2> {
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
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        fn update_from_read(target: &mut BaseStatisticsPart2, read: &io::WrappedFastQRead) {
            //todo: I might want to split this into two threads
            let read_len = read.len();
            if target.per_position_counts.len() <= read_len {
                target
                    .per_position_counts
                    .resize(read_len, PositionCount([0; 5]));
            }
            let seq: &[u8] = read.seq();
            for (ii, base) in seq.iter().enumerate() {
                // using the lookup table is *much* faster than a match
                // and only very slightly slower than using base & 0x7 as index
                // into an array of size 8. And unlike the 0x7 bit trick
                // it is not wrongly mapping non bases to agct
                let idx = BASE_TO_INDEX[*base as usize];
                target.per_position_counts[ii].0[idx as usize] += 1;
            }
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = &mut self.data[tag as usize];
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
        (block, true)
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
                self.data[0].store("base_statistics", &mut contents);
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    let mut local = serde_json::Map::new();
                    self.data[tag as usize].store("base_statistics", &mut local);
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
