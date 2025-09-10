#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{
        TargetPlusAll,
        deser::{bstring_from_string, u8_from_char_or_number},
    },
    dna::TagValue,
};
use anyhow::bail;

use super::super::Step;
use super::{
    apply_in_place_wrapped_with_tag, default_comment_insert_char, default_comment_separator,
    default_region_separator, default_target_read1, format_numeric_for_comment,
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
    #[serde(default = "default_target_read1")]
    target: TargetPlusAll,

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
    fn validate(
        &self,
        input_def: &crate::config::Input,
        output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::super::validate_target_plus_all(self.target, input_def)?;

        match self.target {
            TargetPlusAll::Read1 => {
                if let Some(output) = output_def {
                    if !output
                        .output
                        .as_ref()
                        .map(|x| x.iter().any(|y| y == "read1"))
                        .unwrap_or(false)
                    {
                        bail!(
                            "StoreTagInComment is configured to write comments to Read1, but the output does not contain Read1. Available: {}",
                            output
                                .output
                                .as_ref()
                                .map(|x| x.join(", "))
                                .unwrap_or("none".to_string())
                        );
                    }
                }
            }
            TargetPlusAll::Read2 => {
                if let Some(output) = output_def {
                    if !output
                        .output
                        .as_ref()
                        .map(|x| x.iter().any(|y| y == "read2"))
                        .unwrap_or(false)
                    {
                        bail!(
                            "StoreTagInComment is configured to write comments to Read2, but the output does not contain Read2. Available: {}",
                            output
                                .output
                                .as_ref()
                                .map(|x| x.join(", "))
                                .unwrap_or("none".to_string())
                        );
                    }
                }
            }
            TargetPlusAll::Index1 => {
                if let Some(output) = output_def {
                    if !output
                        .output
                        .as_ref()
                        .map(|x| x.iter().any(|y| y == "index1"))
                        .unwrap_or(false)
                    {
                        bail!(
                            "StoreTagInComment is configured to write comments to Index1, but the output does not contain Index1. Available: {}",
                            output
                                .output
                                .as_ref()
                                .map(|x| x.join(", "))
                                .unwrap_or("none".to_string())
                        );
                    }
                }
            }
            TargetPlusAll::Index2 => {
                if let Some(output) = output_def {
                    if !output
                        .output
                        .as_ref()
                        .map(|x| x.iter().any(|y| y == "index2"))
                        .unwrap_or(false)
                    {
                        bail!(
                            "StoreTagInComment is configured to write comments to Index2, but the output does not contain Index2. Available: {}",
                            output
                                .output
                                .as_ref()
                                .map(|x| x.join(", "))
                                .unwrap_or("none".to_string())
                        );
                    }
                }
            }
            TargetPlusAll::All => {}
        }
        Ok(())
    }

    fn uses_tags(&self) -> Option<Vec<String>> {
        vec![self.label.clone()].into()
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        apply_in_place_wrapped_with_tag(
            self.target,
            &self.label,
            &mut block,
            |read: &mut crate::io::WrappedFastQReadMut, tag_val: &TagValue| {
                let tag_value: Vec<u8> = match tag_val {
                    TagValue::Sequence(hits) => hits.joined_sequence(Some(&self.region_separator)),
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

                store_tag_in_comment(
                    read,
                    self.label.as_bytes(),
                    &tag_value,
                    self.comment_separator,
                    self.comment_insert_char,
                );
            },
        );

        (block, true)
    }
}
