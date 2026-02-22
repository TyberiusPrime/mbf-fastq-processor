use crate::dna::TagValue;
use crate::transformations::prelude::*;
use std::collections::BTreeMap;

use std::sync::OnceLock;

/// Histogram data structure that can handle both String and Numeric tags
#[derive(Debug, Clone)]
pub enum HistogramData {
    /// String values mapped to their counts
    String(BTreeMap<String, usize>),
    /// Numeric values bucketed into bins (value -> count)
    Numeric(BTreeMap<i64, usize>),
    /// Boolean values (false count, true count)
    Bool(usize, usize),
}

impl HistogramData {
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::single_match)]
    pub fn add_value(&mut self, tag_value: &TagValue) {
        match tag_value {
            TagValue::Missing => {
                match self {
                    HistogramData::String(hash_map) => {
                        *hash_map.entry(String::new()).or_insert(0) += 1;
                    }
                    _ => {} // Don't count missing values otherwise.
                }
            }
            TagValue::String(s) => {
                if let HistogramData::String(map) = self {
                    *map.entry(s.to_string()).or_insert(0) += 1;
                } else {
                    unreachable!();
                }
            }
            TagValue::Numeric(n) => {
                // Round to nearest integer for bucketing
                let bucket = n.round() as i64;
                if let HistogramData::Numeric(map) = self {
                    *map.entry(bucket).or_insert(0) += 1;
                } else {
                    unreachable!();
                }
            }
            TagValue::Bool(b) => {
                if let HistogramData::Bool(false_count, true_count) = self {
                    if *b {
                        *true_count += 1;
                    } else {
                        *false_count += 1;
                    }
                } else {
                    unreachable!();
                }
            }
            TagValue::Location(hits) => {
                let s = hits.joined_sequence(Some(b"_"));
                let s = std::str::from_utf8(&s).unwrap_or("").to_string();
                if let HistogramData::String(map) = self {
                    *map.entry(s).or_insert(0) += 1;
                } else {
                    unreachable!();
                }
            }
        }
    }
}

impl From<HistogramData> for serde_json::Value {
    fn from(value: HistogramData) -> Self {
        match value {
            HistogramData::String(map) => map.iter().map(|(k, v)| (k.clone(), *v)).collect(),
            //json only does string keys
            HistogramData::Numeric(map) => map.iter().map(|(k, v)| (k.to_string(), *v)).collect(),

            HistogramData::Bool(false_count, true_count) => {
                let mut map = serde_json::Map::new();
                map.insert("true".into(), true_count.into());
                map.insert("false".into(), false_count.into());
                serde_json::Value::Object(map)
            }
        }
    }
}

#[derive(Clone)]
#[tpd]
#[derive(Debug)]
pub struct _ReportTagHistogram {
    pub report_no: usize,
    pub tag_name: String,
    #[tpd(skip)]
    pub tag_type: OnceLock<TagValueType>,
    #[tpd(skip)]
    pub data: Arc<Mutex<DemultiplexedData<HistogramData>>>,
}

impl _ReportTagHistogram {
    pub fn new(report_no: usize, tag_name: String) -> Self {
        Self {
            report_no,
            tag_name,
            tag_type: OnceLock::new(),
            data: Arc::new(Mutex::new(DemultiplexedData::default())),
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

    fn uses_tags(
        &self,
        tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        if let Some(actual_type) = tags_available.get(&self.tag_name).map(|meta| meta.tag_type) {
            self.tag_type.set(actual_type).expect("Tag type set twice");
        } else {
            return None;
        }
        Some(vec![(
            self.tag_name.clone(),
            &[
                TagValueType::String,
                TagValueType::Bool,
                TagValueType::Numeric,
                TagValueType::Location,
            ],
        )])
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
        let mut data = self.data.lock().expect("Lock poisoned");
        for valid_tag in demultiplex_info.iter_tags() {
            data.insert(
                valid_tag,
                match self
                    .tag_type
                    .get()
                    .expect("Tag type must be set at this point")
                {
                    TagValueType::Location | TagValueType::String => {
                        HistogramData::String(BTreeMap::new())
                    }
                    TagValueType::Numeric => HistogramData::Numeric(BTreeMap::new()),
                    TagValueType::Bool => HistogramData::Bool(0, 0),
                    // _ => {
                    //     return Err(anyhow::anyhow!(
                    //         "ReportTagHistogram does not support tag type {:?}",
                    //         self.tag_type
                    //     ));
                    // }
                },
            );
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
        let mut data = self.data.lock().expect("Lock poisoned");
        // Get the tag values for this tag name if they exist
        if let Some(tag_values) = block.tags.get(&self.tag_name) {
            match demultiplex_info {
                OptDemultiplex::No => {
                    // Without demultiplexing - process all reads
                    let histogram = data
                        .get_mut(&0)
                        .expect("no multiplex data found, but expected");
                    for tag_value in tag_values {
                        histogram.add_value(tag_value);
                    }
                }
                OptDemultiplex::Yes(_) => {
                    // With demultiplexing - process reads by their demultiplex tag
                    if let Some(output_tags) = &block.output_tags {
                        for (read_idx, &demux_tag) in output_tags.iter().enumerate() {
                            if let Some(histogram) = data.get_mut(&demux_tag) {
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

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let data = self.data.lock().expect("Lock poisoned");
        let mut contents = serde_json::Map::new();
        let mut histogram_contents = serde_json::Map::new();
        let histogram_key = self.tag_name.clone();

        match demultiplex_info {
            OptDemultiplex::No => {
                let histogram = data.get(&0).expect("no multiplex data found, but expected");
                histogram_contents.insert(histogram_key, histogram.clone().into());
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                for (tag, name) in &demultiplex_info.tag_to_name {
                    let mut local_histogram_contents = serde_json::Map::new();
                    let barcode_key = name.as_ref().map_or("no-barcode", |x| x.as_str());
                    let histogram = data
                        .get(tag)
                        .expect("no multiplex data found, but expected");
                    local_histogram_contents
                        .insert(histogram_key.clone(), histogram.clone().into());
                    histogram_contents.insert(
                        barcode_key.to_string(),
                        serde_json::Value::Object(local_histogram_contents),
                    );
                }
            }
        }

        contents.insert(
            "histogram".to_string(),
            serde_json::Value::Object(histogram_contents),
        );

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}
