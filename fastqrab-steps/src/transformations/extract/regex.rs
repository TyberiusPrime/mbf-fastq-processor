use std::cell::RefCell;
use std::rc::Rc;

use super::extract_region_tags_using_tags;
use super::extract_string_tags_using_tags;
use crate::transformations::prelude::*;
use bstr::ByteSlice;
use fastqrab_config::{tpd_adapt_bstring, tpd_adapt_regex};

fn regex_replace_with_self() -> BString {
    BString::from("$0")
}

/// Region by regular expression
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Regex {
    #[tpd(with = "tpd_adapt_regex")]
    #[tpd(alias = "pattern")]
    #[tpd(alias = "query")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,

    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub replacement: BString,
    out_label: TagLabel,

    #[tpd(adapt_in_verify(String), alias = "segment")]
    #[schemars(with = "String")]
    source: ResolvedSourceNoAll,
}

impl VerifyIn<PartialConfig> for PartialRegex {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        self.replacement.or_with(regex_replace_with_self);
        self.replacement.verify(|replacement| {
            let group_hunting_regexp =
                regex::bytes::Regex::new("[$]\\d+_").expect("hardcoded regex must compile");
            if group_hunting_regexp.is_match(replacement) {
                Err(ValidationFailure::new(
                    "Replacement string contains a '$1_' style group reference",
                    Some(
                        "This is a footgun: '$1_' is interpreted as group name '1_', not '$1' followed by '_'. Use '${1}_' instead.",
                    ),
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialRegex> {
    fn get_tag_usage(
        &mut self,
        _tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        if let Some(inner) = self.toml_value.value.as_mut() {
            let mut used_tags = Vec::new();
            if let Some(replacement) = inner.replacement.as_ref() {
                let replacement = replacement.clone();
                let re = regex::bytes::Regex::new(r"\[\[(?P<tag>[^\]]+)\]\]")
                    .expect("hardcoded regex must compile");
                let toml_source = Rc::new(RefCell::new((
                    &mut inner.replacement.state,
                    &mut inner.replacement.help,
                )));
                for hit in re.captures_iter(replacement.as_bytes()) {
                    let tag = hit
                        .name("tag")
                        .expect("Regex should always match")
                        .as_bytes();
                    let tag = TagLabel::Normal(
                        std::str::from_utf8(tag)
                            .expect("Tag was not utf8, but toml is always utf8?)")
                            .to_string(),
                    );
                    // we already chek these for 'being present', just like any other
                    // tag.
                    used_tags.push(Some(UsedTag {
                        name: tag,
                        accepted_tag_types: &[
                            TagValueType::String,
                            TagValueType::Location,
                            TagValueType::Numeric((None, None)),
                            TagValueType::Bool,
                        ],
                        toml_source: toml_source.clone(),
                        further_help: None,
                    }));
                }
            }
            let declared_tag = inner.out_label.to_declared_tag({
                if let Some(MustAdapt::PostVerify(source)) = inner.source.as_ref() {
                    match source {
                        ResolvedSourceNoAll::Segment(_segment_index) => TagValueType::Location,
                        ResolvedSourceNoAll::Tag(_tag_label) => TagValueType::String,
                        ResolvedSourceNoAll::Name { .. } => TagValueType::String,
                    }
                } else {
                    TagValueType::Location
                }
            });
            if let Some(MustAdapt::PostVerify(ResolvedSourceNoAll::Tag(tag_name))) =
                inner.source.as_ref()
            {
                used_tags.push(Some(UsedTag {
                    name: tag_name.clone(),
                    accepted_tag_types: &[TagValueType::String, TagValueType::Location],
                    toml_source: Rc::new(RefCell::new((
                        &mut inner.source.state,
                        &mut inner.source.help,
                    ))),
                    further_help: None,
                }));
            }

            Some(TagUsageInfo {
                declared_tag,
                used_tags,
                ..Default::default()
            })
        } else {
            None // cov:excl-line
        }
    }
}

impl Step for Regex {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        fn apply_regexp(
            search: &regex::bytes::Regex,
            replacement: &BString,
            haystack: &[u8],
            read_no: usize,
            block_tags: &IndexMap<TagLabel, TagColumn>,
        ) -> Option<Vec<u8>> {
            let re_hit = search.captures(haystack);
            if let Some(hit) = re_hit {
                let mut out = Vec::new();
                //let g = hit.get(0).expect("Regex should always match");
                hit.expand(&replacement, &mut out);
                for (tag_name, tags) in block_tags {
                    // only those we listed in use_tags.
                    let query = format!("[[{tag_name}]]");
                    let value = tags.to_bstr(read_no);
                    out = out.replace(query, value.as_bytes());
                }
                Some(out)
            } else {
                None
            }
        }
        let source = &self.source;

        match source {
            ResolvedSourceNoAll::Tag(tag_name) => {
                extract_string_tags_using_tags(
                    &mut block,
                    SegmentIndex::first(),
                    &self.out_label,
                    |_read, read_no, block_tags| {
                        // Choose source based on whether it's name or sequence
                        let haystack = block_tags
                            .get(tag_name)
                            .expect("Tag not present?!")
                            .to_bstr(read_no);
                        apply_regexp(
                            &self.search,
                            &self.replacement,
                            &haystack,
                            read_no,
                            block_tags,
                        )
                        .map(|x| x.into())
                    },
                );
            }
            ResolvedSourceNoAll::Name { segment_index, .. } => {
                let block_tags = &block.tags;
                let read_no = block.first_read_sequential_number;
                let mut out = Vec::with_capacity(block.row_count());
                for (ii, read_name) in block
                    .member(segment_index.as_index())
                    .names
                    .iter()
                    .enumerate()
                {
                    // Choose source based on whether it's name or sequence
                    let haystack = read_name;
                    out.push(
                        apply_regexp(
                            &self.search,
                            &self.replacement,
                            haystack,
                            read_no + ii,
                            block_tags,
                        )
                        .map(|x| x.into()),
                    );
                }
                block
                    .tags
                    .insert(self.out_label.clone(), TagColumn::String(out));
            }
            ResolvedSourceNoAll::Segment(segment_index) => {
                extract_region_tags_using_tags(
                    &mut block,
                    *segment_index,
                    &self.out_label,
                    |read_seq, read_no, block_tags| {
                        // Choose source based on whether it's name or sequence
                        let haystack = read_seq;

                        let re_hit = self.search.captures(haystack);
                        if let Some(hit) = re_hit {
                            let mut replacement = Vec::new();
                            let g = hit.get(0).expect("Regex should always match");
                            //dbg!(&self.replacement);
                            hit.expand(&self.replacement, &mut replacement);
                            //dbg!(bstr::BStr::new(&replacement));
                            for (tag_name, tags) in block_tags {
                                // only those we listed in use_tags.
                                let query = format!("[[{tag_name}]]");
                                let value = tags.to_bstr(read_no);
                                // dbg!(&query, &tags[read_no], &value);
                                //  dbg!(bstr::BStr::new(&replacement));
                                replacement = replacement.replace(query, value.as_bytes());
                                //dbg!(bstr::BStr::new(&replacement));
                            }
                            Some(HitDraft {
                                location: Some(HitRegionView {
                                    start: g.start(),
                                    len: g.end() - g.start(),
                                    segment_index: *segment_index,
                                }),
                                sequence: replacement,
                            })
                        } else {
                            None
                        }
                    },
                );
            }
        }
        Ok((block, true))
    }
}
