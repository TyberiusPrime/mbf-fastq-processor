use crate::transformations::prelude::*;

use super::common::PerReadReportData;
use crate::io;

#[derive(Default, Clone)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct _ReportLengthDistribution {
    pub report_no: usize,
    #[tpd(skip)]
    pub data: Arc<Mutex<DemultiplexedData<PerReadReportData<Vec<usize>>>>>,
}

impl _ReportLengthDistribution {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Arc::new(Mutex::new(DemultiplexedData::default())),
        }
    }
}

impl Step for Box<_ReportLengthDistribution> {
    fn transmits_premature_termination(&self) -> bool {
        false
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
        let mut data_lock = self.data.lock().expect("lock poisened");
        for valid_tag in demultiplex_info.iter_tags() {
            data_lock.insert(valid_tag, PerReadReportData::new(input_info));
        }
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn update_from_read(target: &mut Vec<usize>, read: &io::WrappedFastQRead) {
            let read_len = read.len();
            if target.len() <= read_len {
                //println!("Had to resize report buffer, {read_len}");
                target.resize(read_len + 1, 0);
            }
            target[read_len] += 1;
        }
        let mut data: DemultiplexedData<PerReadReportData<Vec<usize>>> =
            DemultiplexedData::default();
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = data
                .entry(tag)
                .or_insert(PerReadReportData::new(input_info));
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
        let mut data_lock = self.data.lock().expect("lock poisened");
        for (tag, report_data) in data {
            let stored = data_lock.get_mut(&tag).expect("tag must exist in data map");
            for (segment_no, (_segment_name, lengths)) in
                report_data.segments.into_iter().enumerate()
            {
                for (len, count) in lengths.into_iter().enumerate() {
                    if stored.segments[segment_no].1.len() <= len {
                        stored.segments[segment_no].1.resize(len + 1, 0);
                    }
                    stored.segments[segment_no].1[len] += count;
                }
            }
        }
        Ok((block, true))
    }

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let data_lock = self.data.lock().expect("lock poisened");
        let mut contents = serde_json::Map::new();
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                data_lock
                    .get(&0)
                    .expect("tag 0 must exist in data map")
                    .store("length_distribution", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        data_lock
                            .get(tag)
                            .expect("tag must exist in data map")
                            .store("length_distribution", &mut local);
                        contents.insert(name.clone(), local.into());
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
