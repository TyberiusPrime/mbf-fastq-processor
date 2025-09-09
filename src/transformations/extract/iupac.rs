#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;

use crate::{
    config::{deser::iupac_from_string, Target},
    dna::Anchor,
    Demultiplexed,
};

use super::super::Step;
use super::extract_tags;

///Extract a IUPAC described sequence from the read. E.g. an adapter.
///Can be at the start (anchor = Left, the end (anchor = Right),
///or anywhere (anchor = Anywhere) within the read.
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[allow(clippy::upper_case_acronyms)]
pub struct IUPAC {
    #[serde(deserialize_with = "iupac_from_string")]
    search: BString,
    pub target: Target,
    anchor: Anchor,
    label: String,
    #[serde(default)] // 0 is fine.
    max_mismatches: u8,
}

impl Step for IUPAC {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
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
            read.find_iupac(&self.search, self.anchor, self.max_mismatches, self.target)
        });

        (block, true)
    }
}
