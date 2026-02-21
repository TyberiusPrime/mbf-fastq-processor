#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;
use crate::{
    config::{deser::tpd_adapt_iupac_bstring},
    dna::Anchor,
};

use super::extract_region_tags;

/// Extract a IUPAC described sequence from the read. E.g. an adapter.
/// Can be at the start (anchor = Left, the end (anchor = Right),
/// or anywhere (anchor = Anywhere) within the read.
/// The search parameter can be either a single IUPAC string or a list of IUPAC strings.
/// If multiple strings are provided, all will be searched and they must be distinct
/// (non-overlapping patterns).
#[derive( Clone, JsonSchema)]
#[tpd]
#[allow(clippy::upper_case_acronyms)]
#[derive( Debug)]
pub struct IUPAC {
    #[tpd(with="tpd_adapt_iupac_bstring")]
    #[tpd(alias="query")]
    #[tpd(alias="pattern")]
    #[schemars(with = "StringOrVecString")]
    search: Vec<BString>,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    anchor: Anchor,
    out_label: String,
    max_mismatches: u8,
}


impl VerifyIn<PartialConfig> for PartialIUPAC {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
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
            self.segment,
            &self.out_label,
            |read| {
                // Try each query pattern and return the first match
                for query in &self.search {
                    if let Some(hit) = read.find_iupac(
                        query,
                        self.anchor,
                        self.max_mismatches,
                        self.segment,
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
