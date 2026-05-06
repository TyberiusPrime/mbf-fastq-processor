use serde_json::json;

use super::common::{BASE_TO_INDEX, PerReadReportData, PositionCount};
use crate::transformations::prelude::*;
use fastqrab_io::io::WrappedFastQRead;

#[derive(Debug, Default, Clone)]
pub struct BaseStatisticsPart2 {
    per_position_counts: Vec<PositionCount>,
}

#[expect(clippy::from_over_into, reason = "Orphan rules")]
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

#[derive(Default, Clone)]
#[tpd(no_verify)]
#[derive(Debug)]
pub struct _ReportBaseStatisticsPart2 {
    pub report_no: usize,
    #[tpd(skip)]
    pub data: Arc<Mutex<DemultiplexedData<PerReadReportData<BaseStatisticsPart2>>>>,
}

impl Partial_ReportBaseStatisticsPart2 {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no: TomlValue::new_ok_unplaced(report_no),
            data: Some(Default::default()),
        }
    }
}
impl TagUser for PartialTaggedVariant<Box<Partial_ReportBaseStatisticsPart2>> {}

//ensure the unsafe below is actually safe.
const _: () = {
    let mut i = 0;
    while i < 256 {
        assert!(
            BASE_TO_INDEX[i] <= 4,
            "BASE_TO_INDEX must not contain values > 4, for the unsafe optimization below to hold"
        );
        i += 1;
    }
};

impl Step for Box<_ReportBaseStatisticsPart2> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    #[mutants::skip] // same result either way, but probably less overhead if we use the per-step
    // lock
    fn needs_serial(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_files: StepOutputFiles,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut data_lock = self.data.lock().expect("data mutex poisoned");
        for valid_tag in demultiplex_info.iter_tags() {
            data_lock.insert(valid_tag, PerReadReportData::new(input_info));
        }
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn update_from_read(target: &mut BaseStatisticsPart2, read: &WrappedFastQRead) {
            let read_len = read.len();
            if target.per_position_counts.len() <= read_len {
                target
                    .per_position_counts
                    .resize(read_len, PositionCount([0; 5]));
            }
            let seq: &[u8] = read.seq();

            for ii in 0..read_len {
                // Optimized: use unsafe to eliminate bounds checking
                // Safety: We just resized to ensure read_len capacity, and we only iterate up to read_len
                // BASE_TO_INDEX always returns 0-4, which is within bounds of the [0; 5] array
                // (and we enforce that with a const assertion above)
                unsafe {
                    let base: u8 = *seq.get_unchecked(ii);
                    let idx = *BASE_TO_INDEX.get_unchecked(base as usize);
                    let counts = target.per_position_counts.get_unchecked_mut(ii);
                    *counts.0.get_unchecked_mut(idx as usize) += 1;
                }
            }
        }
        let mut data_lock = self.data.lock().expect("data mutex poisoned");
        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = data_lock.get_mut(&tag).expect("Lock poisened");
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
        let data_lock = self.data.lock().expect("data mutex poisoned");
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                data_lock
                    .get(&0)
                    .expect("no-multiplex tag found but expected")
                    .store("base_statistics", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        data_lock
                            .get(tag)
                            .expect("no-multiplex tag found but expected")
                            .store("base_statistics", &mut local);
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
