#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{Segment, SegmentIndex, deser::iupac_from_string},
    dna::Anchor,
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
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    anchor: Anchor,
    label: String,
    #[serde(default)] // 0 is fine.
    max_mismatches: u8,
}

impl Step for IUPAC {
    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        extract_tags(
            &mut block,
            self.segment_index.as_ref().unwrap(),
            &self.label,
            |read| {
                read.find_iupac(
                    &self.search,
                    self.anchor,
                    self.max_mismatches,
                    self.segment_index.as_ref().unwrap(),
                )
            },
        );

        Ok((block, true))
    }
}
