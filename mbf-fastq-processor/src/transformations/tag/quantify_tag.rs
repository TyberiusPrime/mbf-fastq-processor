#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::CompressionFormat, transformations::prelude::*};

use bstr::BString;
use std::{collections::BTreeMap, path::Path};

use crate::config::deser::bstring_from_string;
use serde_valid::Validate;

use super::super::{FinalizeReportResult, tag::default_region_separator};

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuantifyTag {
    pub infix: String,
    pub in_label: String,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub collector: DemultiplexedData<BTreeMap<Vec<u8>, usize>>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub output_streams: DemultiplexedOutputFiles,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    region_separator: BString,
}

impl Step for QuantifyTag {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn uses_tags(
        &self,
        _tags_available: &BTreeMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        for tag in demultiplex_info.iter_tags() {
            self.collector.insert(tag, BTreeMap::new());
        }
        self.output_streams = demultiplex_info.open_output_streams(
            output_directory,
            output_prefix,
            &self.infix,
            "qr.json",
            output_ix_separator,
            CompressionFormat::Uncompressed,
            None,
            false,
            false,
            allow_overwrite,
        )?;

        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        Ok((block, true))
        // let collector = &mut self.collector;
        // let hits = block
        //     .tags
        //     .get(&self.in_label)
        //     .expect("Tag not found. Should have been caught in validation");
        // if let Some(demultiplex_tags) = &block.output_tags {
        //     for (tag_val, demultiplex_tag) in hits.iter().zip(demultiplex_tags) {
        //         if let Some(hit) = tag_val.as_sequence() {
        //             *collector
        //                 .get_mut(demultiplex_tag)
        //                 .expect("value must exist in histogram_values")
        //                 .entry(hit.joined_sequence(Some(&self.region_separator)))
        //                 .or_insert(0) += 1;
        //         }
        //     }
        // } else {
        //     for tag_val in hits {
        //         if let Some(hit) = tag_val.as_sequence() {
        //             *collector
        //                 .get_mut(&0)
        //                 .expect("value must exist in histogram_values")
        //                 .entry(hit.joined_sequence(Some(&self.region_separator)))
        //                 .or_insert(0) += 1;
        //         }
        //     }
        // }
        //
        // Ok((block, true))
    }

    fn finalize(
        &mut self,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        use std::io::Write;
        let output_streams = std::mem::take(&mut self.output_streams);
        for (tag, stream) in output_streams.0 {
            if let Some(mut stream) = stream {
                let mut str_collector: Vec<(String, usize)> = self
                    .collector
                    .get(&tag)
                    .expect("value must exist in histogram_values")
                    .iter()
                    .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
                    .collect();
                //sort by count descending, then alphabetically by string
                str_collector.sort_by(|a, b| {
                    b.1.cmp(&a.1)
                        .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
                });
                // we want something that keeps the order
                let str_collector: indexmap::IndexMap<String, usize> =
                    str_collector.into_iter().collect();
                let json = serde_json::to_string_pretty(&str_collector)?;
                stream.write_all(json.as_bytes())?;
            }
        }
        Ok(None)
    }
}
