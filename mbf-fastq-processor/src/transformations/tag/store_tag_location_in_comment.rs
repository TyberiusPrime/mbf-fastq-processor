#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use crate::{
    config::{
        SegmentIndexOrAll, SegmentOrAll,
        deser::{opt_u8_from_char_or_number, u8_from_char_or_number},
    },
    dna::TagValue,
};

use super::{
    apply_in_place_wrapped_with_tag, default_comment_separator, default_segment_all,
    store_tag_in_comment,
};

/// Store currently present tag locations as
/// {tag}_location=target:start-end,target:start-end
///
/// (Aligners often keep only the read name).
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StoreTagLocationInComment {
    in_label: String,

    #[serde(default = "default_segment_all")]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>,

    #[serde(default = "default_comment_separator")]
    #[serde(deserialize_with = "u8_from_char_or_number")]
    comment_separator: u8,

    #[serde(deserialize_with = "opt_u8_from_char_or_number")]
    #[serde(default)]
    comment_insert_char: Option<u8>,
}

impl Step for StoreTagLocationInComment {
    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.comment_insert_char = Some(
            self.comment_insert_char
                .unwrap_or(input_def.options.read_comment_character),
        );

        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let label = format!("{}_location", self.in_label);
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);
        apply_in_place_wrapped_with_tag(
            self.segment_index.as_ref().expect("segment_index must be set during initialization"),
            &self.in_label,
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
                                    input_info.segment_order[location.segment_index.get_index()],
                                    location.start,
                                    location.start + location.len
                                )
                                .as_bytes(),
                            );
                        }
                    }
                }
                let new_name = store_tag_in_comment(
                    read.name(),
                    label.as_bytes(),
                    &seq,
                    self.comment_separator,
                    self.comment_insert_char.expect("comment_insert_char must be set during initialization"),
                );
                //I really don't expect location to fail, but what if the user set's
                //comment_separator to '-'?
                match new_name {
                    Err(err) => {
                        *error_encountered.borrow_mut() = Some(format!("{err}"));
                    }
                    Ok(new_name) => {
                        read.replace_name(new_name);
                    }
                }
            },
        );
        if let Some(error_msg) = error_encountered.borrow().as_ref() {
            return Err(anyhow::anyhow!("{error_msg}"));
        }

        Ok((block, true))
    }
}
