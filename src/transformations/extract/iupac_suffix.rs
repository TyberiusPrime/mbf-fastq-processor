#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{Target, deser::dna_from_string},
    dna::Hits,
};
use anyhow::{Result, bail};

use super::super::{Step, Transformation};
use super::extract_tags;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct IUPACSuffix {
    pub target: Target,
    pub label: String,
    pub min_length: usize,
    pub max_mismatches: usize,
    #[serde(deserialize_with = "dna_from_string")]
    pub query: BString,
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
            let dist = bio::alignment::distance::hamming(&seq[suffix_start..], &query[..prefix_len])
                as usize;
            if dist <= max_mismatches {
                return Some(prefix_len);
            }
        }
        None
    }
}

impl Step for IUPACSuffix {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        if self.max_mismatches > self.min_length {
            bail!("Max mismatches must be <= min length");
        }
        if self.min_length > self.query.len() {
            bail!("Min length must be <= query length");
        }
        super::super::validate_target(self.target, input_def)
    }

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(&mut block, self.target, &self.label, |read| {
            let seq = read.seq();
            if self.query.len() > seq.len() {
                return None;
            }

            if let Some(suffix_len) = Self::longest_suffix_that_is_a_prefix(
                seq,
                &self.query,
                self.max_mismatches,
                self.min_length,
            ) {
                Some(Hits::new(
                    seq.len() - suffix_len,
                    seq.len(),
                    self.target,
                    seq[seq.len() - suffix_len..].to_vec().into(),
                ))
            } else {
                None
            }
        });
        (block, true)
    }
}
