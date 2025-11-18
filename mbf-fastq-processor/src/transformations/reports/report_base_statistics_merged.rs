use crate::transformations::prelude::*;

use super::common::{BASE_TO_INDEX, PerReadReportData, PositionCount, Q20_Q30_LOOKUP, Q_LOOKUP};
use crate::io;
use serde_json::json;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct BaseStatisticsMerged {
    // From Part1
    total_bases: usize,
    q20_bases: usize,
    q30_bases: usize,
    expected_errors_from_quality_curve: Vec<f64>,
    // From Part2
    per_position_counts: Vec<PositionCount>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for BaseStatisticsMerged {
    fn into(self) -> serde_json::Value {
        // Combine both Part1 and Part2 outputs
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
            "total_bases": self.total_bases,
            "q20_bases": self.q20_bases,
            "q30_bases": self.q30_bases,
            "expected_errors_from_quality_curve": self.expected_errors_from_quality_curve,
            "gc_bases": gc_bases,
            "per_position_counts": position_counts
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportBaseStatisticsMerged {
    pub report_no: usize,
    pub data: DemultiplexedData<PerReadReportData<BaseStatisticsMerged>>,
}

impl _ReportBaseStatisticsMerged {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportBaseStatisticsMerged> {
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
        fn update_from_read(target: &mut BaseStatisticsMerged, read: &io::WrappedFastQRead) {
            let read_len = read.len();
            target.total_bases += read_len;

            // Resize both vectors once
            if target.expected_errors_from_quality_curve.len() <= read_len {
                target
                    .expected_errors_from_quality_curve
                    .resize(read_len, 0.0);
            }
            if target.per_position_counts.len() <= read_len {
                target
                    .per_position_counts
                    .resize(read_len, PositionCount([0; 5]));
            }

            let seq = read.seq();
            let qual = read.qual();

            // Use local accumulators for better instruction-level parallelism
            let mut q20_count = 0usize;
            let mut q30_count = 0usize;

            // Single pass through both seq and qual
            for ii in 0..read_len {
                // Base counting (from Part2)
                let idx = BASE_TO_INDEX[seq[ii] as usize];
                target.per_position_counts[ii].0[idx as usize] += 1;

                // Quality counting (from Part1) with lookup table
                let (q20, q30) = Q20_Q30_LOOKUP[qual[ii] as usize];
                q20_count += q20 as usize;
                q30_count += q30 as usize;

                // Expected errors calculation
                let e = Q_LOOKUP[qual[ii] as usize];
                target.expected_errors_from_quality_curve[ii] += e;
            }

            // Update target once at the end
            target.q20_bases += q20_count;
            target.q30_bases += q30_count;
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
