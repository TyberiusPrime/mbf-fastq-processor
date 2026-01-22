use crate::transformations::prelude::*;

use serde_json::json;

#[derive(Debug, Default, Clone, JsonSchema)]
pub struct _ReportCount {
    pub report_no: usize,

    #[schemars(skip)]
    pub data: Arc<Mutex<DemultiplexedData<usize>>>,
}

impl _ReportCount {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: Arc::new(Mutex::new(DemultiplexedData::default())),
        }
    }
}

impl Step for Box<_ReportCount> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    // fn needs_serial(&self) -> bool {
    //     true
    // }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        //if there's a demultiplex step *before* this report,
        //
        let mut data = self.data.lock().expect("mutex poisoned");
        for valid_tag in demultiplex_info.iter_tags() {
            data.insert(valid_tag, 0);
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
        let mut data = self.data.lock().expect("mutex poisoned");
        match demultiplex_info {
            OptDemultiplex::No => {
                *(data.get_mut(&0).expect("tag 0 must exist in data")) += block.len();
            }
            OptDemultiplex::Yes(_) => {
                for tag in block
                    .output_tags
                    .as_ref()
                    .expect("output_tags must be set when demultiplexing")
                {
                    *(data.get_mut(tag).expect("tag must exist in data")) += 1;
                }
            }
        }
        Ok((block, true))
    }

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let data = self.data.lock().expect("mutex poisoned");
        let mut contents = serde_json::Map::new();
        //needs updating for demultiplex
        match demultiplex_info {
            OptDemultiplex::No => {
                contents.insert(
                    "molecule_count".to_string(),
                    (*data.get(&0).expect("tag 0 must exist in data")).into(),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        contents.insert(
                            name.to_string(),
                            json!({
                                "molecule_count": *(data.get(tag).expect("tag must exist in data")),
                            }),
                        );
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
