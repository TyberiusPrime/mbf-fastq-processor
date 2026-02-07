#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::transformations::prelude::*;

use crate::{config::deser::tpd_adapt_iupac_bstring, dna::Anchor};

use super::extract_region_tags;

/// Extract an IUPAC-described sequence while tolerating insertions and deletions.
/// Useful for adapters where small indels are expected.

#[derive(  Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPACWithIndel {
    #[tpd_with(tpd_adapt_iupac_bstring)]
    #[schemars(with = "String")]
    #[tpd_alias("pattern")]
    #[tpd_alias("query")]
    search: BString,
    #[tpd_default]
    segment: Segment,
    #[tpd_skip]
    #[schemars(skip)]
    segment_index: Option<SegmentIndex>,

    anchor: Anchor,
    out_label: String,
    #[tpd_default]
    max_mismatches: usize,
    #[tpd_default]
    max_indel_bases: usize,
    max_total_edits: Option<usize>,
}

impl Step for IUPACWithIndel {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        if self.search.is_empty() {
            anyhow::bail!("search pattern for ExtractIUPACWithIndel cannot be empty");
        }
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
