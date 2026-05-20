use super::extract_region_tags;
use crate::transformations::prelude::*;
use fastqrab_config::{StringOrVecString, dna::Anchor, tpd_adapt_iupac_bstring};

#[derive(Clone, JsonSchema, Debug, Default)]
#[tpd]
enum TieBreak {
    #[default]
    Earliest,
    LeftMost,
    RightMost,
}
/// Extract a IUPAC described sequence from the read. E.g. an adapter.
/// Can be at the start (anchor = Left, the end (anchor = Right),
/// or anywhere (anchor = Anywhere) within the read.
/// The search parameter can be either a single IUPAC string or a list of IUPAC strings.
/// If multiple strings are provided, the first hit wins.
#[derive(Clone, JsonSchema)]
#[tpd]
#[expect(clippy::upper_case_acronyms, reason = "Domain name")]
#[derive(Debug)]
pub struct IUPAC {
    #[tpd(with = "tpd_adapt_iupac_bstring")]
    #[tpd(alias = "query")]
    #[tpd(alias = "pattern")]
    #[schemars(with = "StringOrVecString")]
    search: Vec<BString>,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    anchor: Anchor,
    out_label: TagLabel,
    max_mismatches: u8,
    #[tpd(default)]
    on_tie: TieBreak,
}

impl VerifyIn<PartialConfig> for PartialIUPAC {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialIUPAC> {
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

impl Step for IUPAC {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
            // Try each query pattern and return the first match
            match (&self.on_tie, &self.anchor) {
                (TieBreak::Earliest, _) | (_, Anchor::Left | Anchor::Right) => {
                    for query in &self.search {
                        if let Some(hit) =
                            read.find_iupac(query, self.anchor, self.max_mismatches, self.segment)
                        {
                            return Some(hit);
                        }
                    }
                    return None;
                }
                (TieBreak::LeftMost, Anchor::Anywhere) => {
                    return self
                        .search
                        .iter()
                        .filter_map(|query| {
                            read.find_iupac(query, self.anchor, self.max_mismatches, self.segment)
                        })
                        .min_by_key(|hit| {
                            hit.0[0]
                                .location
                                .as_ref()
                                .map(|x| x.start)
                                .expect("Found iiupac should have had location set")
                        });
                }
                (TieBreak::RightMost, Anchor::Anywhere) => {
                    return self
                        .search
                        .iter()
                        .filter_map(|query| {
                            read.find_iupac(query, self.anchor, self.max_mismatches, self.segment)
                        })
                        .max_by_key(|hit| {
                            hit.0[0]
                                .location
                                .as_ref()
                                .map(|x| x.start)
                                .expect("Found iiupac should have had location set")
                        });
                }
            }
        });

        Ok((block, true))
    }
}
