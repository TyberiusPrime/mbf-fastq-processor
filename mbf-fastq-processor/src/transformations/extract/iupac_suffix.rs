#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::dna::hamming;
use crate::{config::deser::dna_from_string, dna::Hits};

use super::extract_region_tags;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IUPACSuffix {
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub out_label: String,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[serde(deserialize_with = "dna_from_string")]
    #[schemars(with = "String")]
    #[serde(alias = "query")]
    #[serde(alias = "pattern")]
    pub search: BString,
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
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.max_mismatches > self.min_length {
            bail!("Max mismatches must be <= min length");
        }
        if self.min_length > self.search.len() {
            bail!("Min length must be <= query length");
        }
        Ok(())
    }

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
        extract_region_tags(
            &mut block,
            self.segment_index
                .expect("segment_index must be set during initialization"),
            &self.out_label,
            |read| {
                let seq = read.seq();
                if self.search.len() > seq.len() {
                    return None;
                }

                if let Some(suffix_len) = Self::longest_suffix_that_is_a_prefix(
                    seq,
                    &self.search,
                    self.max_mismatches,
                    self.min_length,
                ) {
                    Some(Hits::new(
                        seq.len() - suffix_len,
                        seq.len(),
                        self.segment_index
                            .expect("segment_index must be set during initialization"),
                        seq[seq.len() - suffix_len..].to_vec().into(),
                    ))
                } else {
                    None
                }
            },
        );
        Ok((block, true))
    }
}
