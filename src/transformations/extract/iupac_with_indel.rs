#![allow(clippy::unnecessary_wraps)] // eserde false positives
use anyhow::Result;
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{Segment, SegmentIndex, deser::iupac_from_string},
    dna::Anchor,
};

use super::super::Step;
use super::extract_region_tags;

/// Extract an IUPAC-described sequence while tolerating insertions and deletions.
/// Useful for adapters where small indels are expected.
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPACWithIndel {
    #[serde(deserialize_with = "iupac_from_string")]
    search: BString,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    anchor: Anchor,
    label: String,
    #[serde(default)]
    max_mismatches: usize,
    #[serde(default)]
    max_indel_bases: usize,
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
            self.label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let segment_index = self
            .segment_index
            .expect("segment index should be available after validation");

        extract_region_tags(&mut block, segment_index, &self.label, |read| {
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
