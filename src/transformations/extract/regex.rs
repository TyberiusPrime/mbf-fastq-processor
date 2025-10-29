#![allow(clippy::unnecessary_wraps)] //eserde false positives
use anyhow::Result;
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{
        SegmentOrNameIndex, SegmentSequenceOrName,
        deser::{bstring_from_string, u8_regex_from_string},
    },
    dna::Hits,
};
use anyhow::bail;

use super::super::Step;
use super::extract_tags;

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Regex {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    pub replacement: BString,
    label: String,
    #[serde(alias="segment")]
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
            crate::transformations::TagValueType::Location,
        ))
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let segment_or_name = self.segment_index.unwrap();
        let segment_index = segment_or_name.get_segment_index();

        extract_tags(&mut block, segment_index, &self.label, |read| {
            // Choose source based on whether it's name or sequence
            let source = if segment_or_name.is_name() {
                read.name()
            } else {
                read.seq()
            };

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

        Ok((block, true))
    }
}
