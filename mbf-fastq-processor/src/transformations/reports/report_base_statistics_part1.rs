use crate::transformations::prelude::*;

use super::common::{PHRED33OFFSET, PerReadReportData, Q_LOOKUP};
use crate::io;
use std::path::Path;

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct BaseStatisticsPart1 {
    total_bases: usize,
    q20_bases: usize,
    q30_bases: usize,
    expected_errors_from_quality_curve: Vec<f64>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for BaseStatisticsPart1 {
    fn into(self) -> serde_json::Value {
        serde_json::value::to_value(self).expect("BaseStatisticsPart1 serialization must succeed")
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportBaseStatisticsPart1 {
    pub report_no: usize,
    pub data: DemultiplexedData<PerReadReportData<BaseStatisticsPart1>>,
}

impl _ReportBaseStatisticsPart1 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportBaseStatisticsPart1> {
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
        fn update_from_read(target: &mut BaseStatisticsPart1, read: &io::WrappedFastQRead) {
            let read_len = read.len();
            target.total_bases += read_len;
            if target.expected_errors_from_quality_curve.len() <= read_len {
                target
                    .expected_errors_from_quality_curve
                    .resize(read_len, 0.0);
            }
            let q20_bases = 0;
            let q30_bases = 0;

            for (ii, base) in read.qual().iter().enumerate() {
                if *base >= 20 + PHRED33OFFSET {
                    target.q20_bases += 1;
                    if *base >= 30 + PHRED33OFFSET {
                        target.q30_bases += 1;
                    }
                }
                // averaging phred with the arithetic mean is a bad idea.
                // https://www.drive5.com/usearch/manual/avgq.html
                // I think what we should be reporting are the expected errors
                // this (powf) is very slow, so we use a lookup table
                // let q = base.saturating_sub(PHRED33OFFSET) as f64;
                // let e = 10f64.powf(q / -10.0);
                // % expected value at each position.
                let e = Q_LOOKUP[*base as usize];
                target.expected_errors_from_quality_curve[ii] += e;
            }
            target.q20_bases += q20_bases;
            target.q30_bases += q30_bases;
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = self.data.get_mut(&tag).expect("tag must exist in data");
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
        match &demultiplex_info {
            OptDemultiplex::No => {
                self.data
                    .get(&0)
                    .expect("tag 0 must exist in data")
                    .store("base_statistics", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        self.data
                            .get(tag)
                            .expect("tag must exist in data")
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
