use crate::transformations::prelude::*;

use serde_json::json;
use std::path::Path;

#[derive(Debug, Default, Clone, JsonSchema)]
pub struct _ReportCount {
    pub report_no: usize,

    #[schemars(skip)]
    pub data: DemultiplexedData<usize>,
}

impl _ReportCount {
    pub fn new(report_no: usize) -> Self {
        Self {
            report_no,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportCount> {
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
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        //if there's a demultiplex step *before* this report,
        //
        for valid_tag in demultiplex_info.iter_tags() {
            self.data.insert(valid_tag, 0);
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
        match demultiplex_info {
            OptDemultiplex::No => *(self.data.get_mut(&0).unwrap()) += block.len(),
            OptDemultiplex::Yes(_) => {
                for tag in block.output_tags.as_ref().unwrap() {
                    *(self.data.get_mut(tag).unwrap()) += 1;
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
        match demultiplex_info {
            OptDemultiplex::No => {
                contents.insert(
                    "molecule_count".to_string(),
                    (*self.data.get(&0).unwrap()).into(),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        contents.insert(
                            name.to_string(),
                            json!({
                                "molecule_count": *(self.data.get(tag).unwrap()),
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
