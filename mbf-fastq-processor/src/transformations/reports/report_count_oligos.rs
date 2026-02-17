use crate::transformations::prelude::*;
use memchr::memmem;
use serde_json::{Map, Value};

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct _ReportCountOligos {
    pub report_no: usize,
    pub oligos: Vec<String>,
    #[tpd(skip)]
    #[schemars(skip)]
    pub counts: Arc<Mutex<DemultiplexedData<Vec<usize>>>>,
    pub segment: SegmentOrAll,
    #[tpd(skip)]
    #[schemars(skip)]
    pub segment_index: Option<SegmentIndexOrAll>,
}

impl _ReportCountOligos {
    pub fn new(report_no: usize, oligos: &[String], segment: SegmentOrAll) -> Self {
        let oligos = oligos.to_vec();
        Self {
            report_no,
            oligos,
            counts: Arc::new(Mutex::new(DemultiplexedData::default())),
            segment,
            segment_index: None,
        }
    }
}

impl Step for Box<_ReportCountOligos> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        false
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut counts = self.counts.lock().expect("counts mutex poisoned");
        for valid_tag in demultiplex_info.iter_tags() {
            counts.insert(valid_tag, vec![0; self.oligos.len()]);
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
        let mut blocks = Vec::new();
        match &self
            .segment_index
            .expect("segment_index was not set during validate_segments")
        {
            SegmentIndexOrAll::Indexed(idx) => {
                blocks.push(&block.segments[*idx]);
            }
            SegmentIndexOrAll::All => {
                for segment in &block.segments {
                    blocks.push(segment);
                }
            }
        }
        let mut counts = DemultiplexedData::default();
        for valid_tag in demultiplex_info.iter_tags() {
            counts.insert(valid_tag, vec![0; self.oligos.len()]);
        }

        for read_iter in blocks {
            let mut iter = read_iter.get_pseudo_iter_including_tag(&block.output_tags);
            while let Some((read, demultiplex_tag)) = iter.pseudo_next() {
                let seq = read.seq();

                // Optimized search using memchr for faster substring matching
                for (ii, oligo) in self.oligos.iter().enumerate() {
                    if memmem::find(seq, oligo.as_bytes()).is_some() {
                        counts
                            .get_mut(&demultiplex_tag)
                            .expect("demultiplex tag must exist in counts")[ii] += 1;
                    }
                }
            }
        }

        {
            let mut out_counts = self.counts.lock().expect("counts mutex poisoned");
            for (tag, local_counts) in counts {
                let global_counts = out_counts
                    .get_mut(&tag)
                    .expect("demultiplex tag must exist in counts");
                for (ii, count) in local_counts.iter().enumerate() {
                    global_counts[ii] += count;
                }
            }
        }
        Ok((block, true))
    }
    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let mut contents = Map::new();
        let counts = self.counts.lock().expect("counts mutex poisoned");
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                for (ii, oligo) in self.oligos.iter().enumerate() {
                    contents.insert(
                        oligo.clone(),
                        counts.get(&0).expect("default tag 0 must exist in counts")[ii].into(),
                    );
                }
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let mut local = Map::new();
                        for (ii, oligo) in self.oligos.iter().enumerate() {
                            local.insert(
                                oligo.clone(),
                                counts.get(tag).expect("tag must exist in counts")[ii].into(),
                            );
                        }
                        contents.insert(name.to_string(), local.into());
                    }
                }
            }
        }
        let mut final_contents = Map::new();
        final_contents.insert("count_oligos".to_string(), Value::Object(contents));

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: Value::Object(final_contents),
        }))
    }
}