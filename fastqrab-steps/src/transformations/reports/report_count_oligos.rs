use memchr::memmem;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

use crate::transformations::prelude::*;

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct _ReportCountOligos {
    pub report_no: usize,
    #[schemars(with = "BTreeMap<String, String>")]
    #[tpd(skip)]
    pub oligos: IndexMap<String, BString>,
    #[tpd(skip)]
    #[schemars(skip)]
    pub counts: Arc<Mutex<DemultiplexedData<Vec<usize>>>>,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    pub segment: SegmentIndexOrAll,
}

impl VerifyIn<PartialConfig> for Partial_ReportCountOligos {
    #[mutants::skip]
    // cov:excl-start
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        unreachable!("_ReportCountOligos is only created by ::new, so no verify");
    }
    // cov:excl-stop
}
impl TagUser for PartialTaggedVariant<Box<Partial_ReportCountOligos>> {}

impl Partial_ReportCountOligos {
    pub fn new(
        report_no: usize,
        oligos: IndexMap<String, BString>,
        segment: TomlValue<MustAdapt<String, SegmentIndexOrAll>>,
    ) -> Self {
        Self {
            report_no: TomlValue::new_ok_unplaced(report_no),
            oligos: Some(oligos),
            counts: Some(Default::default()),
            segment,
        }
    }
}

impl Step for Box<_ReportCountOligos> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_files: StepOutputFiles,
        demultiplex_info: &OptDemultiplex,
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
        demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut blocks = Vec::new();
        match &self.segment {
            SegmentIndexOrAll::Indexed(idx) => {
                blocks.push(&block.segments[idx.as_index()]);
            }
            SegmentIndexOrAll::All => {
                for segment in block.segments.iter() {
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
                for (ii, oligo) in self.oligos.values().enumerate() {
                    if memmem::find(seq, oligo.as_slice()).is_some() {
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
        match demultiplex_info {
            OptDemultiplex::No => {
                for (ii, (name, seq)) in self.oligos.iter().enumerate() {
                    let count = counts.get(&0).expect("default tag 0 must exist in counts")[ii];
                    contents.insert(name.clone(), oligo_json_value(name, seq, count));
                }
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, tag_name) in &demultiplex_info.tag_to_name {
                    if let Some(tag_name) = tag_name {
                        let mut local = Map::new();
                        for (ii, (name, seq)) in self.oligos.iter().enumerate() {
                            let count = counts.get(tag).expect("tag must exist in counts")[ii];
                            local.insert(name.clone(), oligo_json_value(name, seq, count));
                        }
                        contents.insert(tag_name.clone(), local.into());
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

/// Produce the JSON value for a single oligo count entry.
/// Always emits `{"sequence": "...", "count": N}`.  When no explicit name was
/// given the caller uses the sequence string as the key, so name and sequence
/// will be the same — but the structure is identical either way.
fn oligo_json_value(_name: &str, seq: &BString, count: usize) -> Value {
    let mut obj = Map::new();
    obj.insert("sequence".to_string(), seq.to_string().into());
    obj.insert("count".to_string(), count.into());
    Value::Object(obj)
}
