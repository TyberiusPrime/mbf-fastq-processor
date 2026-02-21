#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::config::deser::{tpd_adapt_bstring, tpd_adapt_regex};
use crate::transformations::prelude::*;

use crate::{
    config::deser::{bstring_from_string, u8_regex_from_string},
    dna::Hits,
};

use super::extract_region_tags;
use super::extract_string_tags;

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
    out_label: String,

    #[tpd(adapt_in_verify(String), alias = "segment")]
    #[schemars(with = "String")]
    source: SegmentOrNameIndex,
}

impl VerifyIn<PartialConfig> for PartialRegex {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.source.validate_segment(parent);
        self.replacement.or_with(regex_replace_with_self);
        Ok(())
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
        // regex treats  $1_$2 as a group named '1_'
        // and just silently omits it.
        // Let's remove that foot gun. I'm pretty sure you can work around it if
        // you have a group named '1_'...
        let group_hunting_regexp =
            regex::bytes::Regex::new("[$]\\d+_").expect("hardcoded regex must compile");
        if group_hunting_regexp.is_match(&self.replacement) {
            bail!(
                "Replacement string for Regex contains a group reference like  '$1_'. This is a footgun, as it would be interpreted as a group name, not the expected $1 followed by '_' . Please change the replacement string to use ${{1}}_."
            );
        }
        Ok(())
    }

    fn declares_tag_type(&self) -> Option<(String, crate::transformations::TagValueType)> {
        Some((
            self.out_label.clone(),
            if self.source.is_name() {
                crate::transformations::TagValueType::String
            } else {
                crate::transformations::TagValueType::Location
            },
        ))
    }

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
            extract_string_tags(&mut block, segment_index, &self.out_label, |read| {
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
            extract_region_tags(&mut block, segment_index, &self.out_label, |read| {
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
