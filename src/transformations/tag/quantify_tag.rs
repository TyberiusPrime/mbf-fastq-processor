#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use bstr::BString;
use std::{collections::HashMap, io::BufWriter, path::Path};

use crate::config::deser::bstring_from_string;
use serde_valid::Validate;

use super::super::{FinalizeReportResult, tag::default_region_separator};

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuantifyTag {
    pub infix: String,
    pub label: String,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub collector: HashMap<Vec<u8>, usize>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    ix_separator: String,

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with="String")]
    region_separator: BString,
}

impl Step for QuantifyTag {
    fn transmits_premature_termination(&self) -> bool {
        false
    }
    fn needs_serial(&self) -> bool {
        true
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.label.clone(), &[TagValueType::Location])])
    }

    fn configure_output_separator(&mut self, ix_separator: &str) {
        self.ix_separator = ix_separator.to_string();
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
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
        Ok((block, true))
    }

    fn finalize(
        &mut self,
        _input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        let _ = _demultiplex_info;
        use std::io::Write;
        let infix = &self.infix;
        let base = crate::join_nonempty([output_prefix, infix.as_str()], &self.ix_separator);
        let report_file = ex::fs::File::create(output_directory.join(format!("{base}.qr.json")))?;
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
