use std::cell::RefCell;
use std::rc::Rc;

use super::extract_region_tags_using_tags;
use super::extract_string_tags_using_tags;
use crate::transformations::prelude::*;
use bstr::ByteSlice;
use fastqrab_config::{dna::Hits, tpd_adapt_bstring, tpd_adapt_regex};

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
    source: SegmentOrNameIndex,
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

            Some(TagUsageInfo {
                declared_tag: inner.out_label.to_declared_tag(
                    if inner
                        .source
                        .as_ref()
                        .and_then(|x| x.as_ref_post())
                        .is_some_and(|x| x.is_name())
                    {
                        TagValueType::String
                    } else {
                        TagValueType::Location
                    },
                ),
                used_tags: used_tags,
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
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_or_name = self.source;
        let segment_index = segment_or_name.get_segment_index();

        if segment_or_name.is_name() {
            extract_string_tags_using_tags(
                &mut block,
                segment_index,
                &self.out_label,
                |read, read_no, block_tags| {
                    // Choose source based on whether it's name or sequence
                    let source = read.name();

                    let re_hit = self.search.captures(source);
                    if let Some(hit) = re_hit {
                        let mut replacement = Vec::new();
                        //let g = hit.get(0).expect("Regex should always match");
                        hit.expand(&self.replacement, &mut replacement);
                        for (tag_name, tags) in block_tags {
                            // only those we listed in use_tags.
                            let query = format!("[[{tag_name}]]");
                            let value = tags[read_no].to_bstr();
                            replacement = replacement.replace(query, value.as_bytes())
                        }
                        Some(replacement.into())
                    } else {
                        None
                    }
                },
            );
        } else {
            extract_region_tags_using_tags(
                &mut block,
                segment_index,
                &self.out_label,
                |read, read_no, block_tags| {
                    // Choose source based on whether it's name or sequence
                    let source = read.seq();

                    let re_hit = self.search.captures(source);
                    if let Some(hit) = re_hit {
                        let mut replacement = Vec::new();
                        let g = hit.get(0).expect("Regex should always match");
                        //dbg!(&self.replacement);
                        hit.expand(&self.replacement, &mut replacement);
                        //dbg!(bstr::BStr::new(&replacement));
                        for (tag_name, tags) in block_tags {
                            // only those we listed in use_tags.
                            let query = format!("[[{tag_name}]]");
                            let value = tags[read_no].to_bstr();
                            // dbg!(&query, &tags[read_no], &value);
                            //  dbg!(bstr::BStr::new(&replacement));
                            replacement = replacement.replace(query, value.as_bytes());
                            //dbg!(bstr::BStr::new(&replacement));
                        }
                        Some(Hits::new(
                            g.start(),
                            g.end() - g.start(),
                            segment_index,
                            replacement.into(),
                        ))
                    } else {
                        None
                    }
                },
            );
        }
        Ok((block, true))
    }
}
