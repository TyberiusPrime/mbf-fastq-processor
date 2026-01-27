#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{
    config::{
        deser::{bstring_from_string, u8_regex_from_string},
    },
    dna::Hits,
};

use super::extract_region_tags;
use super::extract_string_tags;

fn regex_replace_with_self() -> BString {
    BString::from("$0")
}

/// Region by regular expression
#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Regex {
    #[serde(deserialize_with = "u8_regex_from_string")]
    #[serde(alias = "pattern")]
    #[serde(alias = "query")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    #[serde(default = "regex_replace_with_self")]
    #[schemars(with = "String")]
    pub replacement: BString,
    out_label: String,
    #[serde(alias = "segment")]
    source: String,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<ResolvedSourceNoAll>,
}

impl FromTomlTableNested for Regex {
    fn from_toml_table(_table: &toml_edit::Table, mut helper: TableErrorHelper) -> TomlResult<Self>
    where
        Self: Sized,
    {
        let search: TomlResult<(&str, String)> =
            helper.get_alias(&["search", "query", "pattern"][..]);
        let replacement: TomlResult<Option<BString>> = helper.get_opt("replacement");
        let out_label = helper.get_tag("out_label");
        let resolved_source = helper.get_source_no_all(&["source", "segment"][..], true);

        helper.deny_unknown()?;
        let (source, resolved_source) = resolved_source?;

        // regex treats  $1_$2 as a group named '1_'
        // and just silently omits it.
        // Let's remove that foot gun. I'm pretty sure you can work around it if
        // you have a group named '1_'...
        if let Ok(Some(replacement)) = replacement.as_ref() {
            let group_hunting_regexp =
                regex::bytes::Regex::new("[$]\\d+_").expect("hardcoded regex must compile");
            if group_hunting_regexp.is_match(&replacement) {
                return helper.add_err_by_key("replacement", 
                "Replacement string for Regex contains a group reference like  '$1_'.",
                ".his is a footgun, as it would be interpreted as a group name, not the expected $1 followed by '_' . Please change the replacement string to use ${{1}}_."
        );
            }
        }
        let (search_key, search) = search?;
        let search = regex::bytes::Regex::new(&search)
            .or_else(|e| helper.add_err_by_key(search_key, "Invalid regex", &format!("{}", e)))?;

        Ok(Regex {
            search: search,
            replacement: replacement?.unwrap_or_else(|| regex_replace_with_self()),
            source: source,
            segment_index: Some(resolved_source),
            out_label: out_label?,
        })
    }
}

impl Step for Regex {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            crate::transformations::TagValueType::String, //TODO: tag support
                                                          // if self
                                                          //     .segment_index
                                                          //     .expect("segment_index must be set during initialization")
                                                          //     .is_name()
                                                          // {
                                                          //     crate::transformations::TagValueType::String
                                                          // } else {
                                                          //     crate::transformations::TagValueType::Location
                                                          // },
        ))
    }

    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        match self
            .segment_index
            .as_ref()
            .expect("segment_index must be set during initialization")
        {
            ResolvedSourceNoAll::Segment(segment_index) => {
                extract_region_tags(&mut block, *segment_index, &self.out_label, |read| {
                    // Choose source based on whether it's name or sequence
                    let source = read.seq();

                    let re_hit = self.search.captures(source);
                    if let Some(hit) = re_hit {
                        let mut replacement = Vec::new();
                        let g = hit.get(0).expect("Regex should always match");
                        hit.expand(&self.replacement, &mut replacement);
                        Some(Hits::new(
                            g.start(),
                            g.end() - g.start(),
                            *segment_index,
                            replacement.into(),
                        ))
                    } else {
                        None
                    }
                });
            }
            ResolvedSourceNoAll::Tag(_) => todo!(),
            ResolvedSourceNoAll::Name {
                segment_index,
                split_character: _, //todo
            } => {
                extract_string_tags(&mut block, *segment_index, &self.out_label, |read| {
                    // Choose source based on whether it's name or sequence
                    let source = read.name();

                    let re_hit = self.search.captures(source);
                    if let Some(hit) = re_hit {
                        let mut replacement = Vec::new();
                        //let g = hit.get(0).expect("Regex should always match");
                        hit.expand(&self.replacement, &mut replacement);
                        Some(replacement.into())
                    } else {
                        None
                    }
                });
            }
        }

        Ok((block, true))
    }
}
