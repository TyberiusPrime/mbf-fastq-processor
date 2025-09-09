#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;
use std::{collections::HashMap, io::BufWriter, path::Path};

use crate::{Demultiplexed, config::deser::bstring_from_string};
use anyhow::Result;
use serde_valid::Validate;

use super::super::{FinalizeReportResult, Step, tag::default_region_separator};

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct QuantifyTag {
    pub infix: String,
    pub label: String,

    #[serde(skip)]
    pub collector: HashMap<Vec<u8>, usize>,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,
}

impl Step for QuantifyTag {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let collector = &mut self.collector;
        let hits = block
            .tags
            .as_ref()
            .expect("No tags in block: bug")
            .get(&self.label)
            .expect("Tag not found. Should have been caught in validation");
        for tag_val in hits {
            if let Some(hit) = tag_val.as_sequence() {
                *collector
                    .entry(hit.joined_sequence(Some(&self.region_separator)))
                    .or_insert(0) += 1;
            }
        }
        (block, true)
    }

    fn finalize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        use std::io::Write;
        let infix = &self.infix;
        let report_file = std::fs::File::create(
            output_directory.join(format!("{output_prefix}_{infix}.qr.json")),
        )?;
        let mut bufwriter = BufWriter::new(report_file);
        let mut str_collector: Vec<(String, usize)> = self
            .collector
            .iter()
            .map(|(k, v)| (String::from_utf8_lossy(k).to_string(), *v))
            .collect();
        //sort by count descending, then alphabetically by string
        str_collector.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });
        // we want something that keeps the order
        let str_collector: indexmap::IndexMap<String, usize> = str_collector.into_iter().collect();
        let json = serde_json::to_string_pretty(&str_collector)?;
        bufwriter.write_all(json.as_bytes())?;
        Ok(None)
    }
}
