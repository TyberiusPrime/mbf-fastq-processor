#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;
use crate::{
    config::{Segment, SegmentIndex, deser::iupac_string_or_list},
    dna::Anchor,
};

use super::extract_region_tags;

/// Extract a IUPAC described sequence from the read. E.g. an adapter.
/// Can be at the start (anchor = Left, the end (anchor = Right),
/// or anywhere (anchor = Anywhere) within the read.
/// The search parameter can be either a single IUPAC string or a list of IUPAC strings.
/// If multiple strings are provided, all will be searched and they must be distinct
/// (non-overlapping patterns).
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPAC {
    #[serde(deserialize_with = "iupac_string_or_list")]
    #[serde(alias = "query")]
    #[serde(alias = "pattern")]
    #[schemars(with = "StringOrVecString")]
    search: Vec<BString>,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    anchor: Anchor,
    out_label: String,
    max_mismatches: u8,
}

// Schema helper for string or list of strings
#[derive(JsonSchema)]
#[allow(dead_code)]
#[serde(untagged)]
enum StringOrVecString {
    String(String),
    Vec(Vec<String>),
}

impl Step for IUPAC {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        extract_region_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read| {
                // Try each query pattern and return the first match
                for query in &self.search {
                    if let Some(hit) = read.find_iupac(
                        query,
                        self.anchor,
                        self.max_mismatches,
                        self.segment_index
                            .expect("segment_index must be set during initialization"),
                    ) {
                        return Some(hit);
                    }
                }
                None
            },
        );

        Ok((block, true))
    }
}
