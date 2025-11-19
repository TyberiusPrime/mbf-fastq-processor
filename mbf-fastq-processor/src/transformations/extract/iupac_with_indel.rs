#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::transformations::prelude::*;

use crate::{config::deser::iupac_from_string, dna::Anchor};

use super::extract_region_tags;

/// Extract an IUPAC-described sequence while tolerating insertions and deletions.
/// Useful for adapters where small indels are expected.
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPACWithIndel {
    /// What we are searching (IUPAC string)
    #[serde(deserialize_with = "iupac_from_string")]
    #[schemars(with = "String")]
    search: BString,
    /// Any of your input segments (default: read1)
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    /// Left | Right | Anywhere - Where to search.
    /// Left only matches at the start of the read, Right at the end, Anywhere searches the entire read.
    anchor: Anchor,
    /// Label to store the extracted tag under
    out_label: String,
    /// How many mismatches are allowed (default: 0)
    #[serde(default)]
    max_mismatches: usize,
    /// How many inserted or deleted bases are allowed in total (default: 0)
    #[serde(default)]
    max_indel_bases: usize,
    /// Optional overall edit budget (mismatches + indels)
    #[serde(default)]
    max_total_edits: Option<usize>,
}

impl Step for IUPACWithIndel {
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
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self
            .segment_index
            .expect("segment index should be available after validation");

        extract_region_tags(&mut block, segment_index, &self.out_label, |read| {
            read.find_iupac_with_indel(
                &self.search,
                self.anchor,
                self.max_mismatches,
                self.max_indel_bases,
                self.max_total_edits,
                segment_index,
            )
        });

        Ok((block, true))
    }
}
