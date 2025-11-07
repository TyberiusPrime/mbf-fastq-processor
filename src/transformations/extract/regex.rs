#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;

use crate::{
    config::{
        SegmentOrNameIndex, SegmentSequenceOrName,
        deser::{bstring_from_string, u8_regex_from_string},
    },
    dna::Hits,
};

use super::extract_region_tags;
use super::extract_string_tags;

fn regex_replace_with_self() -> BString {
    BString::from("$0")
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Regex {
    #[serde(deserialize_with = "u8_regex_from_string")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    #[serde(default = "regex_replace_with_self")]
    #[schemars(with = "String")]
    pub replacement: BString,
    label: String,
    #[serde(alias = "segment")]
    source: SegmentSequenceOrName,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentOrNameIndex>,
}

impl Step for Regex {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        // regex treats  $1_$2 as a group named '1_'
        // and just silently omits it.
        // Let's remove that foot gun. I'm pretty sure you can work around it if
        // you have a group named '1_'...
        let group_hunting_regexp = regex::bytes::Regex::new("[$]\\d+_").unwrap();
        if group_hunting_regexp.is_match(&self.replacement) {
            bail!(
                "Replacement string for Regex contains a group reference like  '$1_'. This is a footgun, as it would be interpreted as a group name, not the expected $1 followed by '_' . Please change the replacement string to use ${{1}}_."
            );
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.source.validate(input_def)?);
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.label.clone(),
            if self.segment_index.unwrap().is_name() {
                crate::transformations::TagValueType::String
            } else {
                crate::transformations::TagValueType::Location
            },
        ))
    }

    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let segment_or_name = self.segment_index.unwrap();
        let segment_index = segment_or_name.get_segment_index();

        if segment_or_name.is_name() {
            extract_string_tags(&mut block, segment_index, &self.label, |read| {
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
        } else {
            extract_region_tags(&mut block, segment_index, &self.label, |read| {
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
                        segment_index,
                        replacement.into(),
                    ))
                } else {
                    None
                }
            });
        }

        Ok((block, true))
    }
}
