#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::config::deser::tpd_adapt_dna_bstring;
use crate::transformations::prelude::*;

use crate::dna::hamming;
use crate::dna::Hits;

use super::extract_region_tags;

/// Extract a IUPAC sequence (or a prefix of it) at the end of a read into a tag.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct IUPACSuffix {
    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndex,

    pub out_label: String,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[tpd(with = "tpd_adapt_dna_bstring")]
    #[schemars(with = "String")]
    #[tpd(alias = "query")]
    #[tpd(alias = "pattern")]
    pub search: BString,
}

impl VerifyIn<PartialConfig> for PartialIUPACSuffix {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.max_mismatches.verify(|v| {
            if *v > 255 {
                Err(ValidationFailure::new(
                    "max_mismatches must be <= 255",
                    Some("Set to a value between 0 and 255."),
                ))
            } else {
                Ok(())
            }
        });
        self.min_length.verify(|v| {
            if *v == 0 {
                Err(ValidationFailure::new(
                    "min_length must be > 0",
                    Some("Set to a positive integer."),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl IUPACSuffix {
    #[allow(clippy::cast_possible_truncation)]
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
            let dist = hamming(&seq[suffix_start..], &query[..prefix_len]) as usize;
            if dist <= max_mismatches {
                return Some(prefix_len);
            }
        }
        None
    }
}

impl Step for IUPACSuffix {
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
        extract_region_tags(&mut block, self.segment, &self.out_label, |read| {
            let seq = read.seq();

            //cheap empty range if read length too short no need for explicit check
            Self::longest_suffix_that_is_a_prefix(
                seq,
                &self.search,
                self.max_mismatches,
                self.min_length,
            )
            .map(|suffix_len| {
                Hits::new(
                    seq.len() - suffix_len,
                    seq.len(),
                    self.segment,
                    seq[seq.len() - suffix_len..].to_vec().into(),
                )
            })
        });
        Ok((block, true))
    }
}
