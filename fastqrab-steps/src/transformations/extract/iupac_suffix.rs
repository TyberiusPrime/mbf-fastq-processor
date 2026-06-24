use super::extract_region_tags_from_seq;
use crate::transformations::prelude::*;
use fastqrab_config::{dna::iupac_hamming_distance, tpd_adapt_iupac_bstring};

/// Extract a IUPAC sequence (or a prefix of it) at the end of a read into a tag.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct IUPACSuffix {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    pub out_label: TagLabel,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[tpd(with = "tpd_adapt_iupac_bstring")]
    #[schemars(with = "String")]
    #[tpd(alias = "query")]
    #[tpd(alias = "pattern")]
    pub search: BString,
}

impl VerifyIn<PartialConfig> for PartialIUPACSuffix {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        let ml_span = self.min_length.span();
        self.min_length.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "min_length must be > 0",
                    Some("Set to a positive integer."),
                ))
            } else {
                if let Some(search) = self.search.as_ref()
                    && *v > search.len()
                {
                    let spans = vec![
                        (
                            ml_span,
                            "Too large, can not be longer than the search pattern".to_string(),
                        ),
                        (self.search.span(), "or this too short?".to_string()),
                    ];
                    self.search.state = TomlValueState::Custom { spans };
                    self.search.help = Some(
                        "min_length cannot be greater than the length of the search pattern"
                            .to_string(),
                    );
                }
                Ok(())
            }
        });
        Ok(())
    }
}

impl IUPACSuffix {
    fn longest_suffix_that_is_a_prefix(
        seq: &[u8],
        query: &[u8],
        max_mismatches: usize,
        min_length: usize,
    ) -> Option<usize> {
        assert!(min_length >= 1);
        let max_len = std::cmp::min(seq.len(), query.len());
        for prefix_len in (min_length..=max_len).rev() {
            let suffix_start = seq.len() - prefix_len;
            let dist = iupac_hamming_distance(&query[..prefix_len], &seq[suffix_start..]);
            if dist <= max_mismatches {
                return Some(prefix_len);
            }
        }
        None
    }
}

impl TagUser for PartialTaggedVariant<PartialIUPACSuffix> {
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

impl Step for IUPACSuffix {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        extract_region_tags_from_seq(&mut block, self.segment, &self.out_label, |seq| {
            //cheap empty range if read length too short no need for explicit check
            #[expect(
                clippy::cast_possible_truncation,
                reason = "lengths are guaranteed to be within u32 range"
            )]
            Self::longest_suffix_that_is_a_prefix(
                seq,
                &self.search,
                self.max_mismatches,
                self.min_length,
            )
            .map(|suffix_len| (seq.len() - suffix_len) as u32..seq.len() as u32)
        });
        Ok((block, true))
    }
}
