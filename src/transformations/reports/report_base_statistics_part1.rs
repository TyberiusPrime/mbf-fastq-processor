use super::super::{FinalizeReportResult, InputInfo, Step};
use super::common::{PHRED33OFFSET, PerReadReportData, Q_LOOKUP};
use crate::{
    demultiplex::{DemultiplexInfo, Demultiplexed},
    io,
};
use anyhow::Result;
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
        serde_json::value::to_value(self).unwrap()
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportBaseStatisticsPart1 {
    pub report_no: usize,
    pub data: Vec<PerReadReportData<BaseStatisticsPart1>>,
}

impl _ReportBaseStatisticsPart1 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Vec::default(),
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
        fn update_from_read(target: &mut BaseStatisticsPart1, read: &io::WrappedFastQRead) {
            //todo: I might want to split this into two threads
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
                // I think what we should be reporting is the
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
