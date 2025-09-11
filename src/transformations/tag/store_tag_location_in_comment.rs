#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::{
    config::{deser::u8_from_char_or_number, SegmentIndexOrAll, SegmentOrAll},
    dna::TagValue,
    Demultiplexed,
};
use anyhow::Result;

use super::super::Step;
use super::{
    apply_in_place_wrapped_with_tag, default_comment_insert_char, default_comment_separator,
    default_segment_all, store_tag_in_comment,
};

/// Store currently present tag locations as
/// {tag}_location=target:start-end,target:start-end
///
/// (Aligners often keep only the read name).
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTaglocationInComment {
    label: String,

    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,

    #[serde(default = "default_comment_separator")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_separator: u8,
    #[serde(default = "default_comment_insert_char")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_insert_char: u8,
}

impl Step for StoreTaglocationInComment {
    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let label = format!("{}_location", self.label);
        apply_in_place_wrapped_with_tag(
            self.segment_index.as_ref().unwrap(),
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, tag_val: &TagValue| {
                let mut seq: Vec<u8> = Vec::new();
                if let Some(hits) = tag_val.as_sequence() {
                    let mut first = true;
                    for hit in &hits.0 {
                        if let Some(location) = hit.location.as_ref() {
                            if !first {
                                seq.push(b',');
                            }
                            first = false;
                            seq.extend_from_slice(
                                format!(
                                    "{}:{}-{}",
                                    location.segment_index.get_name(),
                                    location.start,
                                    location.start + location.len
                                )
                                .as_bytes(),
                            );
                        }
                    }
                }
                store_tag_in_comment(
                    read,
                    label.as_bytes(),
                    &seq,
                    self.comment_separator,
                    self.comment_insert_char,
                );
            },
        );

        (block, true)
    }
}
