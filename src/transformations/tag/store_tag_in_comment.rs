#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{
        SegmentIndexOrAll, SegmentOrAll,
        deser::{bstring_from_string, u8_from_char_or_number},
    },
    dna::TagValue,
    transformations::TagValueType,
};
use anyhow::bail;

use super::super::Step;
use super::{
    apply_in_place_wrapped_with_tag, default_comment_insert_char, default_comment_separator,
    default_region_separator, default_segment_all, format_numeric_for_comment,
    store_tag_in_comment,
};

/// Store currently present tags as comments on read names.
/// Comments are key=value pairs, separated by `comment_separator`
/// which defaults to '|'.
/// They get inserted at the first `comment_insert_char`,
/// which defaults to space. The `comment_insert_char` basically moves
/// to the right.
///
/// That means a read name like
/// @ERR12828869.501 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
/// becomes
/// @ERR12828869.501|key=value|key2=value2 A00627:18:HGV7TDSXX:3:1101:10502:5274/1
///
/// This way, your added tags will survive STAR alignment.
/// (STAR always cuts at the first space, and by default also on /)
///
/// (If the `comment_insert_char` is not present, we simply add at the right)
///
///
/// Be default, comments are only placed on Read1.
/// If you need them somewhere else, or an all reads, change the target (to "All")
#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct StoreTagInComment {
    label: String,
    #[serde(default = "default_segment_all")]
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

    #[serde(default = "default_region_separator")]
    #[serde(deserialize_with = "bstring_from_string")]
    region_separator: BString,
}

impl Step for StoreTagInComment {
    fn validate_others(
        &self,
        input_def: &crate::config::Input,
        output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        match self.segment_index.as_ref().unwrap() {
            SegmentIndexOrAll::All => {}
            SegmentIndexOrAll::Indexed(idx) => {
                let name = &input_def.get_segment_order()[*idx];
                let available_output_segments = {
                    if let Some(output_def) = output_def {
                        let mut res = Vec::new();
                        if let Some(interleaved) = &output_def.interleave {
                            res.extend(interleaved.iter().cloned());
                        }
                        if let Some(output) = &output_def.output {
                            res.extend(output.iter().cloned());
                        }
                        res
                    } else {
                        //bail!("Using StoreTagInComment when not outputting anything is pointless"
                        //actually, the only time this will happen is in a report only run.
                        //and if the user requests it (maybe commented out the output?)
                        //who are we to complain
                        vec![name.to_string()]
                    }
                };
                if !available_output_segments.contains(name) {
                    bail!(
                        "StoreTagInComment is configured to write comments to '{name}', but the output does not contain '{name}'. Available: {available_output_segments:?}",
                    );
                }
            }
        }
        if self.label.bytes().any(|x| x == b'=') {
            bail!(
                "Tag labels cannot contain '='. Observed label: '{}'",
                self.label
            );
        }
        for (desc, k) in &[
            ("comment separator", self.comment_separator),
            ("comment insert char", self.comment_insert_char),
        ] {
            if self.label.bytes().any(|x| x == *k) {
                bail!(
                    "Tag labels cannot contain {desc}: '{}' Observed label: '{}'",
                    BString::new(vec![*k]),
                    self.label
                );
            }
        }

        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.label.clone(), &[TagValueType::Any])])
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);
        apply_in_place_wrapped_with_tag(
            self.segment_index.as_ref().unwrap(),
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, tag_val: &TagValue| {
                let tag_value: Vec<u8> = match tag_val {
                    TagValue::Sequence(hits) => hits.joined_sequence(Some(&self.region_separator)),
                    TagValue::String(value) => value.to_vec(),
                    TagValue::Numeric(n) => format_numeric_for_comment(*n).into_bytes(),
                    TagValue::Bool(n) => {
                        if *n {
                            "1".into()
                        } else {
                            "0".into()
                        }
                    }
                    TagValue::Missing => Vec::new(),
                };

                let new_name = store_tag_in_comment(
                    read.name(),
                    self.label.as_bytes(),
                    &tag_value,
                    self.comment_separator,
                    self.comment_insert_char,
                );
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
