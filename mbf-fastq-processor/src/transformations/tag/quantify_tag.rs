#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{config::{CompressionFormat, deser::tpd_adapt_bstring}, transformations::prelude::*};

use std::{collections::BTreeMap};

use crate::config::deser::bstring_from_string;

use super::super::tag::default_region_separator;

type QuantifyTagCollector = Arc<Mutex<DemultiplexedData<BTreeMap<Vec<u8>, usize>>>>;

/// Write a histogram of tag values to a JSON file.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct QuantifyTag {
    pub infix: String,
    pub in_label: String,

    #[schemars(with = "String")]
    #[tpd(with="tpd_adapt_bstring")]
    region_separator: BString,

    #[tpd(skip)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub collector: QuantifyTagCollector,

    #[tpd(skip)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub output_streams: Arc<Mutex<DemultiplexedOutputFiles>>,

}

impl VerifyIn<PartialConfig> for PartialQuantifyTag {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.region_separator.or_with(default_region_separator);
        Ok(())
    }
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
        _tags_available: &IndexMap<String, TagMetadata>,
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
        let mut collector = self.collector.lock().expect("Lock poisoned");
        for tag in demultiplex_info.iter_tags() {
            collector.insert(tag, BTreeMap::new());
        }
        self.output_streams = Arc::new(Mutex::new(demultiplex_info.open_output_streams(
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
        )?));

        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut collector = self.collector.lock().expect("Lock poisoned");
        let hits = block
            .tags
            .get(&self.in_label)
            .expect("Tag not found. Should have been caught in validation");
        if let Some(demultiplex_tags) = &block.output_tags {
            for (tag_val, demultiplex_tag) in hits.iter().zip(demultiplex_tags) {
                if let Some(hit) = tag_val.as_sequence() {
                    *collector
                        .get_mut(demultiplex_tag)
                        .expect("value must exist in histogram_values")
                        .entry(hit.joined_sequence(Some(&self.region_separator)))
                        .or_insert(0) += 1;
                }
            }
        } else {
            for tag_val in hits {
                if let Some(hit) = tag_val.as_sequence() {
                    *collector
                        .get_mut(&0)
                        .expect("value must exist in histogram_values")
                        .entry(hit.joined_sequence(Some(&self.region_separator)))
                        .or_insert(0) += 1;
                }
            }
        }

        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        use std::io::Write;
        let collector = self.collector.lock().expect("Lock poisoned");
        let output_streams = self.output_streams.lock().expect("Lock poisoned").take();
        for (tag, stream) in output_streams {
            if let Some(mut stream) = stream {
                let mut str_collector: Vec<(String, usize)> = collector
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
