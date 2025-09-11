use super::super::{
    FinalizeReportResult, FragmentEntry, FragmentEntryForCuckooFilter, InputInfo, OurCuckCooFilter,
    Step, reproducible_cuckoofilter,
};
use crate::demultiplex::{DemultiplexInfo, Demultiplexed};
use anyhow::Result;
use std::path::Path;

#[derive(Default, Debug, Clone)]
pub struct DuplicateFragmentCountData {
    duplicate_count: usize,
    duplication_filter: Option<OurCuckCooFilter<FragmentEntryForCuckooFilter>>,
}

#[allow(clippy::from_over_into)]
impl Into<serde_json::Value> for DuplicateFragmentCountData {
    fn into(self) -> serde_json::Value {
        self.duplicate_count.into()
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportDuplicateFragmentCount {
    pub report_no: usize,
    //that is per read1/read2...
    pub data: Vec<DuplicateFragmentCountData>,
    pub debug_reproducibility: bool,
}

impl Step for Box<_ReportDuplicateFragmentCount> {
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
        demultiplex_info: &Demultiplexed,
    ) -> Result<Option<DemultiplexInfo>> {
        let (initial_capacity, false_positive_probability) = if self.debug_reproducibility {
            (100, 0.1)
        } else {
            (1_000_000, 0.01)
        };

        for _ in 0..=(demultiplex_info.max_tag()) {
            self.data.push(DuplicateFragmentCountData {
                duplicate_count: 0,
                duplication_filter: Some(reproducible_cuckoofilter(
                    42,
                    initial_capacity,
                    false_positive_probability,
                )),
            });
        }
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        {
            let mut block_iter = block.get_pseudo_iter();
            let pos = 0;
            while let Some(molecule) = block_iter.pseudo_next() {
                let inner: Vec<_> = molecule.segments.iter().map(|x| x.seq()).collect();
                let seq = FragmentEntry(&inner);
                // passing in this complex/reference type into the cuckoo_filter
                // is a nightmare.
                let tag = block.output_tags.as_ref().map_or(0, |x| x[pos]);
                let target = &mut self.data[tag as usize];
                if target.duplication_filter.as_ref().unwrap().contains(&seq) {
                    target.duplicate_count += 1;
                    println!(
                        "Found a duplicate: {}",
                        std::str::from_utf8(molecule.segments[0].name()).unwrap()
                    );
                } else {
                    target.duplication_filter.as_mut().unwrap().insert(&seq);
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
                contents.insert(
                    "fragment_duplicate_count".to_string(),
                    self.data[0].duplicate_count.into(),
                );
            }

            Demultiplexed::Yes(demultiplex_info) => {
                for (tag, barcode) in demultiplex_info.iter_outputs() {
                    let mut local = serde_json::Map::new();
                    local.insert(
                        "fragment_duplicate_count".to_string(),
                        self.data[tag as usize].duplicate_count.into(),
                    );
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
