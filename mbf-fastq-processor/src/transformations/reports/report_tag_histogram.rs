use crate::dna::TagValue;
use crate::transformations::prelude::*;

use super::super::FinalizeReportResult;
use std::collections::HashMap;
use std::path::Path;

/// Histogram data structure that can handle both String and Numeric tags
#[derive(Debug, Default, Clone)]
pub enum HistogramData {
    #[default]
    Empty,
    /// String values mapped to their counts
    String(HashMap<String, usize>),
    /// Numeric values bucketed into bins (value -> count)
    Numeric(HashMap<i64, usize>),
    /// Boolean values (false count, true count)
    Bool(usize, usize),
}

impl HistogramData {
    pub fn add_value(&mut self, tag_value: &TagValue) {
        match tag_value {
            TagValue::Missing => {
                // Don't count missing values
            }
            TagValue::String(s) => {
                if let HistogramData::String(ref mut map) = self {
                    *map.entry(s.to_string()).or_insert(0) += 1;
                } else if matches!(self, HistogramData::Empty) {
                    let mut map = HashMap::new();
                    map.insert(s.to_string(), 1);
                    *self = HistogramData::String(map);
                }
            }
            TagValue::Numeric(n) => {
                // Round to nearest integer for bucketing
                let bucket = n.round() as i64;
                if let HistogramData::Numeric(ref mut map) = self {
                    *map.entry(bucket).or_insert(0) += 1;
                } else if matches!(self, HistogramData::Empty) {
                    let mut map = HashMap::new();
                    map.insert(bucket, 1);
                    *self = HistogramData::Numeric(map);
                }
            }
            TagValue::Bool(b) => {
                if let HistogramData::Bool(ref mut false_count, ref mut true_count) = self {
                    if *b {
                        *true_count += 1;
                    } else {
                        *false_count += 1;
                    }
                } else if matches!(self, HistogramData::Empty) {
                    if *b {
                        *self = HistogramData::Bool(0, 1);
                    } else {
                        *self = HistogramData::Bool(1, 0);
                    }
                }
            }
            TagValue::Location(_) => {
                // For location tags, just count presence/absence as boolean
                if let HistogramData::Bool(ref mut false_count, ref mut true_count) = self {
                    *true_count += 1;
                } else if matches!(self, HistogramData::Empty) {
                    *self = HistogramData::Bool(0, 1);
                }
            }
        }
    }
}

impl From<HistogramData> for serde_json::Value {
    fn from(value: HistogramData) -> Self {
        match value {
            HistogramData::Empty => serde_json::json!({}),
            HistogramData::String(map) => {
                let mut sorted: Vec<_> = map.into_iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
                serde_json::json!({
                    "type": "string",
                    "counts": sorted.into_iter().map(|(k, v)| {
                        serde_json::json!({"label": k, "count": v})
                    }).collect::<Vec<_>>()
                })
            }
            HistogramData::Numeric(map) => {
                let mut sorted: Vec<_> = map.into_iter().collect();
                sorted.sort_by_key(|a| a.0);
                serde_json::json!({
                    "type": "numeric",
                    "counts": sorted.into_iter().map(|(k, v)| {
                        serde_json::json!({"value": k, "count": v})
                    }).collect::<Vec<_>>()
                })
            }
            HistogramData::Bool(false_count, true_count) => {
                serde_json::json!({
                    "type": "bool",
                    "counts": [
                        {"label": "false", "count": false_count},
                        {"label": "true", "count": true_count}
                    ]
                })
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct _ReportTagHistogram {
    pub report_no: usize,
    pub tag_name: String,
    pub data: DemultiplexedData<HistogramData>,
}

impl _ReportTagHistogram {
    pub fn new(report_no: usize, tag_name: String) -> Self {
        Self {
            report_no,
            tag_name,
            data: DemultiplexedData::default(),
        }
    }
}

impl Step for Box<_ReportTagHistogram> {
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
        for valid_tag in demultiplex_info.iter_tags() {
            self.data.insert(valid_tag, HistogramData::default());
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
        // Get the tag values for this tag name if they exist
        if let Some(tag_values) = block.tags.get(&self.tag_name) {
            match demultiplex_info {
                OptDemultiplex::No => {
                    // Without demultiplexing - process all reads
                    let histogram = self.data.get_mut(&0).unwrap();
                    for tag_value in tag_values {
                        histogram.add_value(tag_value);
                    }
                }
                OptDemultiplex::Yes(_) => {
                    // With demultiplexing - process reads by their demultiplex tag
                    if let Some(output_tags) = &block.output_tags {
                        for (read_idx, &demux_tag) in output_tags.iter().enumerate() {
                            if let Some(histogram) = self.data.get_mut(&demux_tag) {
                                let tag_value = &tag_values[read_idx];
                                histogram.add_value(tag_value);
                            }
                        }
                    }
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
        let histogram_key = format!("histogram_{}", self.tag_name);

        match demultiplex_info {
            OptDemultiplex::No => {
                let histogram = self.data.get(&0).unwrap();
                contents.insert(
                    histogram_key,
                    histogram.clone().into(),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    if let Some(name) = name {
                        let histogram = self.data.get(tag).unwrap();
                        contents.insert(
                            name.to_string(),
                            serde_json::json!({
                                histogram_key: histogram.clone()
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
