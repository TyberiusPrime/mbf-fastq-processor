use super::{format_numeric_for_comment, store_tag_in_comment};
use crate::transformations::prelude::*;
use fastqrab_config::{
    TagValueType, default_comment_insert_char, default_comment_separator, default_region_separator,
    tpd_adapt_bstring, tpd_adapt_u8_from_byte_or_char,
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
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct StoreTagInComment {
    #[tpd(adapt_in_verify(String), alias = "in_label")]
    in_labels: Vec<TagLabel>,

    #[tpd(adapt_in_verify(String))]
    #[schemars(with = "String")]
    segment: SegmentIndexOrAll,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub comment_separator: u8,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    comment_insert_char: u8,

    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    region_separator: BString,
}

impl VerifyIn<PartialConfig> for PartialStoreTagInComment {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(input_def) = parent.input.as_ref()
            && !input_def.get_segment_order().is_empty()
        {
            self.segment.or(SegmentIndexOrAll::All); //TODO: verify if this is what we want to do,
            //or if the above comment on it being only on segment 0 is corect.
        }
        self.segment.validate_segment(parent);
        self.comment_separator.or_with(default_comment_separator);
        self.region_separator.or_with(default_region_separator);
        self.comment_insert_char.or_with(|| {
            parent
                .input
                .as_ref()
                .and_then(|x| x.options.as_ref())
                .and_then(|x| x.read_comment_character.as_ref())
                .copied()
                .unwrap_or_else(default_comment_insert_char)
        });

        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialStoreTagInComment> {
    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut()
            && let Some(in_labels) = inner.in_labels.value.as_mut()
        {
            for tag in in_labels.iter_mut() {
                tag.validate_incoming_tag_label(tags_available, segment_order);
                if let Some(tag_name) = tag.as_ref().and_then(|x| x.as_ref_post()) {
                    let str_tag_name: &str = tag_name.as_ref();
                    //truly paranoia, tag labels are a-zA-Z0-9_, but the user might have set the
                    //separators/insert chars to one of them, I suppose
                    if let Some(sep) = inner.comment_separator.as_ref().copied()
                        && str_tag_name.bytes().any(|x| x == sep)
                    {
                        let spans = vec![
                            (
                                tag.span(),
                                "Tag label contains comment_separator".to_string(),
                            ),
                            (
                                inner.comment_separator.span(),
                                "comment_separator defined here".to_string(),
                            ),
                        ];
                        inner.comment_separator.state = TomlValueState::Custom { spans };
                        inner.comment_separator.help = Some(format!(
                            "Tag labels cannot contain the comment_separator '{}'. You probably want a comment separator that's not in a-zA-Z0-9",
                            BString::new(vec![sep])
                        ));
                    }
                    if let Some(ins) = inner.comment_insert_char.as_ref().copied()
                        && str_tag_name.bytes().any(|x| x == ins)
                    {
                        let spans = vec![
                            (
                                tag.span(),
                                "Tag label contains comment_insert_char".to_string(),
                            ),
                            (
                                inner.comment_insert_char.span(),
                                "comment_insert_char defined here".to_string(),
                            ),
                        ];
                        inner.comment_insert_char.state = TomlValueState::Custom { spans };
                        inner.comment_insert_char.help = Some(format!(
                            "Tag labels cannot contain the comment_insert_char '{}'. You probably want an insert char that's not in a-zA-Z0-9",
                            BString::new(vec![ins])
                        ));
                    }
                }
            }

            let mut used_tags = Vec::new();
            for in_label in in_labels {
                in_label.validate_incoming_tag_label(tags_available, segment_order);
                used_tags.push(in_label.to_used_tag(&[
                    TagValueType::Bool,
                    TagValueType::String,
                    TagValueType::Location,
                    TagValueType::Numeric((None, None)),
                ]));
            }
            Some(TagUsageInfo {
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for StoreTagInComment {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let tag_list = &self.in_labels;

        for segment_idx in block.iter_segment_indices(self.segment) {
            let mut new_names = StringPodBuilder::new(); //TODO: profile if prealloc is sensible
            let old_names = &block.segments[segment_idx].names;
            for read_idx in 0..block.len() {
                let name = old_names.get(read_idx);
                let mut new_name: Result<Vec<u8>> = Ok(name.to_vec());
                for tag_label in tag_list {
                    let tag_col = block.tags.get(tag_label).expect("Tags were checked before");
                    let tag_value = tag_col.to_bstr(
                        read_idx,
                        format_numeric_for_comment,
                        Some(self.region_separator.as_ref()),
                    );
                    let tag_name: &str = tag_label.as_ref();
                    new_name = store_tag_in_comment(
                        new_name.as_ref().expect("Err would have left the loop"),
                        tag_name.as_bytes(),
                        &tag_value,
                        self.comment_separator,
                        self.comment_insert_char,
                    );
                    if let Err(err) = new_name {
                        bail!(err)
                    }
                }
                if let Ok(new_name) = new_name {
                    new_names.push(&new_name);
                }
            }
            block.segments[segment_idx].names = new_names.finish();
        }

        Ok((block, true))
    }
}
