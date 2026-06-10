use bstr::ByteSlice;

use super::extract_region_tags_from_seq;
use crate::transformations::prelude::*;
use fastqrab_config::{dna::Anchor, tpd_adapt_iupac_bstring};
use fastqrab_dna::dna::find_iupac_with_indel;

/// Extract an IUPAC-described sequence while tolerating insertions and deletions.
/// Useful for adapters where small indels are expected.

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
//#[expect(clippy::upper_case_acronyms, reason="Domain name")]
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
    out_label: TagLabel,
    #[tpd(default)]
    max_mismatches: usize,
    #[tpd(default)]
    max_indel_bases: usize,
    max_total_edits: Option<usize>,
}

impl VerifyIn<PartialConfig> for PartialIUPACWithIndel {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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

impl TagUser for PartialTaggedVariant<PartialIUPACWithIndel> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(TagValueType::Location),
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for IUPACWithIndel {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_index = self.segment;

        extract_region_tags_from_seq(&mut block, segment_index, &self.out_label, |read_seq| {
            find_iupac_with_indel(
                read_seq,
                self.search.as_bstr(),
                self.anchor,
                self.max_mismatches,
                self.max_indel_bases,
                self.max_total_edits,
            )
        });

        Ok((block, true))
    }
}
