use super::extract_region_tags_from_seq;
use crate::transformations::prelude::*;
use fastqrab_config::{StringOrVecString, dna::Anchor, tpd_adapt_iupac_bstring};
use fastqrab_dna::dna::find_iupac;

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
    max_anchor_distance: usize,
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
        if let Some(Anchor::Anywhere) = self.anchor.as_ref()
            && self.max_anchor_distance.as_ref().is_some()
        {
            let spans = vec![
                (
                    self.max_anchor_distance.span.clone(),
                    "Incomptable with anchor = anywhere".to_string(),
                ),
                (
                    self.anchor.span.clone(),
                    "Incomptable with max_anchor_distance set".to_string(),
                ),
            ];
            self.max_anchor_distance.state = TomlValueState::Custom { spans };
            self.max_anchor_distance.help = Some("Either choose a different anchor, or remove max_anchor_distance, depending on your eneeds".to_string());
        }
        self.max_anchor_distance.or(0);
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
            // Record the segment this location tag lives on, so a conditional
            // `Swap` on that segment can forget it (a per-read swap would split it
            // across two segments).
            let segment = inner
                .segment
                .as_ref()
                .and_then(|m| m.as_ref_post())
                .copied();
            Some(TagUsageInfo {
                declared_tag: inner
                    .out_label
                    .to_declared_tag(TagValueType::Location)
                    .map(|dt| match segment {
                        Some(seg) => dt.with_segment(seg),
                        None => dt,
                    }),
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
        extract_region_tags_from_seq(&mut block, self.segment, &self.out_label, |read_seq| {
            // Try each query pattern and return the first match
            match &self.on_tie {
                TieBreak::Earliest => {
                    for query in &self.search {
                        if let Some(hit) = find_iupac(
                            read_seq,
                            query,
                            self.anchor,
                            self.max_mismatches,
                            self.max_anchor_distance,
                        ) {
                            return Some(hit);
                        }
                    }
                    return None;
                }
                TieBreak::LeftMost => {
                    return self
                        .search
                        .iter()
                        .filter_map(|query| {
                            find_iupac(
                                read_seq,
                                query,
                                self.anchor,
                                self.max_mismatches,
                                self.max_anchor_distance,
                            )
                        })
                        .min_by_key(|hit| hit.start);
                }
                TieBreak::RightMost => {
                    return self
                        .search
                        .iter()
                        .filter_map(|query| {
                            find_iupac(
                                read_seq,
                                query,
                                self.anchor,
                                self.max_mismatches,
                                self.max_anchor_distance,
                            )
                        })
                        .max_by_key(|hit| hit.start);
                }
            }
        });

        Ok((block, true))
    }
}
