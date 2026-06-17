use super::common::PerReadReportData;
use crate::transformations::prelude::*;
use crate::transformations::tag::calculate_filter_capacity;

#[derive(Default, Debug, Clone)]
pub struct DuplicateCountData {
    duplicate_count: usize,
    duplication_filter: Option<OurCuckCooFilter<[u8]>>,
}

#[expect(clippy::from_over_into, reason = "Orphan rules")]
impl Into<serde_json::Value> for DuplicateCountData {
    fn into(self) -> serde_json::Value {
        self.duplicate_count.into()
    }
}

#[derive(Default, Clone, Debug)]
#[tpd(no_verify)]
pub struct _ReportDuplicateCount {
    pub report_no: usize,
    //that is per read1/read2...
    pub debug_reproducibility: bool,

    #[tpd(skip)]
    pub data_per_segment: Arc<Mutex<DemultiplexedData<PerReadReportData<DuplicateCountData>>>>,
    #[tpd(skip)]
    pub initial_filter_capacity: Arc<Mutex<Option<usize>>>,
}

impl Partial_ReportDuplicateCount {
    pub fn new(report_no: usize, debug_reproducibility: TomlValue<bool>) -> Self {
        Self {
            report_no: TomlValue::new_ok_unplaced(report_no),
            debug_reproducibility,
            data_per_segment: Some(Default::default()),
            initial_filter_capacity: Some(Default::default()),
        }
    }
}

impl TagUser for PartialTaggedVariant<Box<Partial_ReportDuplicateCount>> {}

impl Step for Box<_ReportDuplicateCount> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    #[mutants::skip]
    fn needs_serial(&self) -> bool {
        //for the init
        true
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_files: StepOutputFiles,
        demultiplex_info: &OptDemultiplex,
        _input_files: &mut StepInputFiles,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Initialize data structures but not the filters (those are initialized in apply)
        let mut data_lock = self.data_per_segment.lock().expect("lock poisened");
        for valid_tag in demultiplex_info.iter_tags() {
            let mut data_per_read = Vec::new();
            for segment_name in &input_info.segment_order {
                data_per_read.push((
                    segment_name.clone(),
                    DuplicateCountData {
                        duplicate_count: 0,
                        duplication_filter: None, // Initialized in apply() on first block
                    },
                ));
            }
            data_lock.insert(
                valid_tag,
                PerReadReportData {
                    segments: data_per_read,
                },
            );
        }
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn update_from_read(target: &mut DuplicateCountData, seq: &BStr) {
            if target
                .duplication_filter
                .as_ref()
                .expect("duplication_filter must be set during initialization")
                .contains(seq)
            {
                target.duplicate_count += 1;
            } else {
                target
                    .duplication_filter
                    .as_mut()
                    .expect("duplication_filter must be set during initialization")
                    .insert(seq);
            }
        }
        // Initialize filters on first block using dynamic sizing
        let mut data_lock = self.data_per_segment.lock().expect("lock poisened");
        if block.block_no() == 1 {
            let false_positive_probability = if self.debug_reproducibility {
                0.1 // cov:excl-line
            } else {
                0.01
            };
            let capacity = calculate_filter_capacity(
                *self.initial_filter_capacity.lock().expect("lock poisened"),
                input_info,
                demultiplex_info.len(),
            );

            self.initial_filter_capacity
                .lock()
                .expect("lock poisened")
                .replace(capacity);

            for tag in demultiplex_info.iter_tags() {
                let output = data_lock
                    .get_mut(&tag)
                    .expect("Tag should have been checked during init?");
                for (_segment_name, data) in &mut output.segments {
                    data.duplication_filter = Some(reproducible_cuckoofilter(
                        42,
                        capacity,
                        false_positive_probability,
                    ));
                }
            }
        }

        for tag in demultiplex_info.iter_tags() {
            // no need to capture no-barcode if we're
            // not outputing it
            let output = data_lock
                .get_mut(&tag)
                .expect("tag must exist in data_per_read");

            for (ii, segment) in block.segments.iter().enumerate() {
                let storage = &mut output.segments[ii].1;
                //Todo: refactor all these repetitions of iter...
                let iter: Box<dyn Iterator<Item = &BStr>> = match &block.output_tags {
                    Some(output_tags) => Box::new(
                        segment
                            .seq_quals
                            .iter_seq()
                            .zip(output_tags.iter())
                            .filter_map(
                                |(read, read_tag)| if *read_tag == tag { Some(read) } else { None },
                            ),
                    ),
                    None => Box::new(segment.seq_quals.iter_seq()),
                };

                for seq in iter {
                    update_from_read(storage, &seq);
                }
            }
        }
        Ok((block, true))
    }

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        let data_lock = self.data_per_segment.lock().expect("lock poisened");

        // Add filter capacity information if available
        // yes, these get set by _ReportDuplicateCount and _ReportFragmentDuplicateCount.
        if let Some(capacity) = *self.initial_filter_capacity.lock().expect("lock poisened") {
            contents.insert(
                "initial_filter_capacity".to_string(),
                serde_json::Value::Number(capacity.into()),
            );
        }
        let actual_filter_capacity = data_lock
            .values()
            .next()
            .and_then(|data| data.segments.first())
            .and_then(|(_name, data)| data.duplication_filter.as_ref())
            .map(scalable_cuckoo_filter::ScalableCuckooFilter::capacity)
            .expect("Could not retrieve filter capacity? Bug");
        contents.insert(
            "actual_filter_capacity".to_string(),
            serde_json::Value::Number(actual_filter_capacity.into()),
        );

        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                data_lock
                    .get(&0)
                    .expect("tag 0 must exist in data_per_read")
                    .store("duplicate_count", &mut contents);
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = serde_json::Map::new();
                        data_lock
                            .get(tag)
                            .expect("tag must exist in data_per_read")
                            .store("duplicate_count", &mut local);
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
