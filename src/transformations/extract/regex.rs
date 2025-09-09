#![allow(clippy::unnecessary_wraps)] //eserde false positives
use bstr::BString;

use crate::{
    Demultiplexed,
    config::{
        Target,
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
    pub target: Target,
}

impl Step for Regex {
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[super::super::Transformation],
        _this_transforms_index: usize,
    ) -> anyhow::Result<()> {
        super::super::validate_target(self.target, input_def)?;
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

    fn sets_tag(&self) -> Option<String> {
        Some(self.label.clone())
    }

    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        extract_tags(&mut block, self.target, &self.label, |read| {
            let re_hit = self.search.captures(read.seq());
            if let Some(hit) = re_hit {
                let mut replacement = Vec::new();
                let g = hit.get(0).expect("Regex should always match");
                hit.expand(&self.replacement, &mut replacement);
                Some(Hits::new(
                    g.start(),
                    g.end() - g.start(),
                    self.target,
                    replacement.into(),
                ))
            } else {
                None
            }
        });

        (block, true)
    }
}
