#![allow(clippy::unnecessary_wraps)]
use crate::transformations::prelude::*;

use crate::{
    config::deser::tpd_adapt_u8_from_byte_or_char, dna::TagValue,
    transformations::prelude::ValidateSegment,
};

use super::{
    apply_in_place_wrapped_with_tag, default_comment_insert_char, default_comment_separator,
    store_tag_in_comment,
};

/// Store currently present tag locations as
/// {tag}_location=target:start-end,target:start-end
///
/// (Aligners often keep only the read name).
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct StoreTagLocationInComment {
    in_label: String,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    comment_separator: u8,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    comment_insert_char: u8,
}

impl VerifyIn<PartialConfig> for PartialStoreTagLocationInComment {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if self.segment.is_missing() {
            self.segment.value = Some(MustAdapt::PostVerify(SegmentIndexOrAll::All));
            self.segment.state = TomlValueState::Ok;
        }
        self.segment.validate_segment(parent);
        self.comment_separator.or_with(default_comment_separator);
        self.comment_insert_char
            .or_with(default_comment_insert_char);
        Ok(())
    }
}

impl Step for StoreTagLocationInComment {
    fn uses_tags(
        &self,
        _tags_available: &IndexMap<String, TagMetadata>,
    ) -> Option<Vec<(String, &[TagValueType])>> {
        Some(vec![(self.in_label.clone(), &[TagValueType::Location])])
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let label = format!("{}_location", self.in_label);
        let error_encountered = std::cell::RefCell::new(Option::<String>::None);
        apply_in_place_wrapped_with_tag(
            &self.segment,
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
                    self.comment_insert_char,
                );
                //I really don't expect location to fail, but what if the user set's
                //comment_separator to '-'?
                match new_name {
                    Err(err) => {
                        *error_encountered.borrow_mut() = Some(format!("{err}"));
                    }
                    Ok(new_name) => {
                        read.replace_name(&new_name);
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
