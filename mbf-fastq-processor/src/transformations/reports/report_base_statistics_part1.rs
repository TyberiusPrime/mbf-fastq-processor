use crate::transformations::prelude::*;

use super::common::{PerReadReportData, Q_LOOKUP, Q20_Q30_LOOKUP};
use crate::io;

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
        serde_json::value::to_value(self).expect("Failed to serialize?")
    }
}

#[derive( Default, Clone)]
#[tpd]
#[derive(Debug)]
pub struct _ReportBaseStatisticsPart1 {
    pub report_no: usize,
    #[tpd_skip]
    pub data: Arc<Mutex<DemultiplexedData<PerReadReportData<BaseStatisticsPart1>>>>,
}

impl _ReportBaseStatisticsPart1 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Arc::new(Mutex::new(DemultiplexedData::default())),
        }
    }
}

impl Step for Box<_ReportBaseStatisticsPart1> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    fn needs_serial(&self) -> bool {
        true //todo: technically untrue, we could multic core & assemble this?
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
                .lock()
                .expect("data poisened?")
                .insert(valid_tag, PerReadReportData::new(input_info));
        }
        Ok(None)
    }

    fn apply(
        &self,
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

            // Use local accumulators for better instruction-level parallelism
            let mut q20_count = 0usize;
            let mut q30_count = 0usize;

            for (ii, base) in read.qual().iter().enumerate() {
                // Use lookup table to eliminate branches
                let (q20, q30) = Q20_Q30_LOOKUP[*base as usize];
                q20_count += q20 as usize;
                q30_count += q30 as usize;

                // Expected errors calculation
                let e = Q_LOOKUP[*base as usize];
                target.expected_errors_from_quality_curve[ii] += e;
            }

            // Update target once at the end
            target.q20_bases += q20_count;
            target.q30_bases += q30_count;
        }
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputting it
            let mut data_lock = self.data.lock().expect("data poisened");
            let output = data_lock
                .get_mut(&tag)
                .expect("demultiplex tag not in data, but expected");
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

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        //needs updating for demultiplex
        match &demultiplex_info {
            OptDemultiplex::No => {
                self.data
                    .lock()
                    .expect("data poisened")
                    .get(&0)
                    .expect("no-demultiplex tag missing in data, but expected")
                    .store("base_statistics", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                let data_lock = self.data.lock().expect("data poisened");
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        data_lock
                            .get(tag)
                            .expect("no-demultiplex tag missing in data, but expected")
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
