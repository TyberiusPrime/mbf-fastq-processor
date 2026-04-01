use std::collections::BTreeMap;

use crate::transformations::prelude::*;

/// Histogram data structure that can handle both String and Numeric tags
#[derive(Debug, Clone)]
pub enum HistogramData {
    /// String values mapped to their counts
    String(BTreeMap<String, usize>),
    /// Numeric values bucketed into bins (value -> count)
    Integer(BTreeMap<i64, usize>),
    ZeroToOne(BTreeMap<NonNaN, usize>),
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
                    _ => {} // cov:excl-line
                            // Don't count missing values otherwise.
                }
            }
            TagValue::String(s) => {
                if let HistogramData::String(map) = self {
                    *map.entry(s.to_string()).or_insert(0) += 1;
                } else {
                    // cov:excl-start
                    unreachable!();
                    // cov:excl-stop
                }
            }
            TagValue::Numeric(n) => {
                // Round to nearest integer for bucketing
                match self {
                    HistogramData::Integer(map) => {
                        let bucket = n.round() as i64;
                        *map.entry(bucket).or_insert(0) += 1;
                    }
                    HistogramData::ZeroToOne(map) => {
                        let bucket = (n * 100.).round() / 100.0;
                        let bucket: NonNaN = bucket
                            .try_into()
                            .expect("NaN value for histogram - not supported");
                        *map.entry(bucket).or_insert(0) += 1;
                    }
                    _ => {
                        // cov:excl-start
                        unreachable!();
                        // cov:excl-stop
                    }
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
                    // cov:excl-start
                    unreachable!();
                    // cov:excl-stop
                }
            }
            TagValue::Location(hits) => {
                let s = hits.joined_sequence(Some(b"_"));
                let s = std::str::from_utf8(&s).unwrap_or("").to_string();
                if let HistogramData::String(map) = self {
                    *map.entry(s).or_insert(0) += 1;
                } else {
                    // cov:excl-start
                    unreachable!();
                    // cov:excl-stop
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
            HistogramData::Integer(map) => map.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            HistogramData::ZeroToOne(map) => {
                map.iter().map(|(k, v)| (format!("{:.2}", k), *v)).collect()
            }

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
#[tpd(no_verify)]
#[derive(Debug)]
pub struct _ReportTagHistogram {
    pub report_no: usize,
    pub tag_name: TagLabel,
    #[tpd(skip)]
    pub tag_type: TagValueType,
    #[tpd(skip)]
    pub data: Arc<Mutex<DemultiplexedData<HistogramData>>>,
}

impl Partial_ReportTagHistogram {
    pub fn new(report_no: usize, tag_name: TomlValue<TagLabel>) -> Self {
        Self {
            report_no: TomlValue::new_ok_unplaced(report_no),
            tag_name,
            tag_type: None,
            data: Some(Default::default()),
        }
    }
}

impl TagUser for PartialTaggedVariant<Box<Partial_ReportTagHistogram>> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            if let Some(tag_meta) =
                tags_available.get(inner.tag_name.as_ref().expect("parent was ok?"))
            {
                inner.tag_type = Some(tag_meta.tag_type);
            } else {
                //no need to set it, missing tag will fail before the 'tag_type not set in verify'
                //if that's happening at all for our dynamically generated one.
            }
            Some(TagUsageInfo {
                used_tags: vec![inner.tag_name.to_used_tag(&[
                    TagValueType::String,
                    TagValueType::Numeric((None, None)),
                    TagValueType::Bool,
                    TagValueType::Location,
                ])],
                ..Default::default()
            })
        } else {
            None
        }
    }
}

impl Step for Box<_ReportTagHistogram> {
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    #[mutants::skip] // makes on difference to the outcome where we block 
    // (only once by pipeline, or in self.data.lock()),
    // but better for structure if the pipeline knows about it
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
        let mut data = self.data.lock().expect("Lock poisoned");
        for valid_tag in demultiplex_info.iter_tags() {
            data.insert(
                valid_tag,
                match self.tag_type {
                    TagValueType::Location | TagValueType::String => {
                        HistogramData::String(BTreeMap::new())
                    }
                    TagValueType::Numeric((lower, upper)) => {
                        if lower == Some(NonNaN::new(0.0).expect("Can't fail"))
                            && upper == Some(NonNaN::new(1.0).expect("can't fail"))
                        {
                            HistogramData::ZeroToOne(BTreeMap::new())
                        } else {
                            HistogramData::Integer(BTreeMap::new())
                        }
                    }
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
                    } // cov:excl-line
                }
            }
        } // cov:excl-line
        Ok((block, true))
    }

    fn finalize(&self, demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let data = self.data.lock().expect("Lock poisoned");
        let mut contents = serde_json::Map::new();
        let histogram_key = self.tag_name.clone();

        match demultiplex_info {
            OptDemultiplex::No => {
                let histogram = data.get(&0).expect("no multiplex data found, but expected");
                let mut histogram_contents = serde_json::Map::new();
                histogram_contents
                    .insert(histogram_key.as_ref().to_string(), histogram.clone().into());
                contents.insert(
                    "histogram".to_string(),
                    serde_json::Value::Object(histogram_contents),
                );
            }

            OptDemultiplex::Yes(demultiplex_info) => {
                // Place histogram nested inside each barcode bucket so that after
                // pipeline_workpool wraps everything in "multiplexed", the histogram
                // lives alongside molecule_count inside the per-bucket object and the
                // HTML template can find it via addSectionTable().
                for (tag, name) in &demultiplex_info.tag_to_name {
                    let barcode_key = name.as_ref().map_or("no-barcode", |x| x.as_str());
                    let histogram = data
                        .get(tag)
                        .expect("no multiplex data found, but expected");
                    let mut inner = serde_json::Map::new();
                    inner.insert(histogram_key.as_ref().to_string(), histogram.clone().into());
                    let mut barcode_contents = serde_json::Map::new();
                    barcode_contents
                        .insert("histogram".to_string(), serde_json::Value::Object(inner));
                    contents.insert(
                        barcode_key.to_string(),
                        serde_json::Value::Object(barcode_contents),
                    );
                }
            }
        }

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}
