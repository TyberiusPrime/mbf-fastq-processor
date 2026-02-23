#![allow(clippy::unnecessary_wraps)] // eserde false positives
use crate::transformations::prelude::*;

use crate::{config::deser::tpd_adapt_iupac_bstring, dna::Anchor};

use super::extract_region_tags;

/// Extract an IUPAC-described sequence while tolerating insertions and deletions.
/// Useful for adapters where small indels are expected.

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPACWithIndel {
    #[tpd(with = "tpd_adapt_iupac_bstring")]
    #[schemars(with = "String")]
    #[tpd(alias = "pattern")]
    #[tpd(alias = "query")]
    search: BString,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    anchor: Anchor,
    out_label: String,
    #[tpd(default)]
    max_mismatches: usize,
    #[tpd(default)]
    max_indel_bases: usize,
    max_total_edits: Option<usize>,
}

impl VerifyIn<PartialConfig> for PartialIUPACWithIndel {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.search.verify(|v| {
            if v.is_empty() {
                return Err(ValidationFailure::new(
                    "Must contain at least one letter (base)",
                    None,
                ));
            }
            Ok(())
        });

        Ok(())
    }
}

impl Step for IUPACWithIndel {
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
        let segment_index = self.segment;

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
