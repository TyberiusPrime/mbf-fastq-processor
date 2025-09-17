#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
use crate::{
    config::{
        SegmentIndex,
        deser::{bstring_from_string, u8_regex_from_string},
    },
    demultiplex::Demultiplexed,
    transformations::apply_in_place_wrapped,
};
use bstr::{BString, ByteSlice};

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Rename {
    #[serde(deserialize_with = "u8_regex_from_string")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    pub replacement: BString,
}

impl Step for Rename {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let handle_name = |read: &mut crate::io::WrappedFastQReadMut| {
            let name = read.name();
            let new_name = self
                .search
                .replace_all(name, self.replacement.as_bytes())
                .into_owned();
            read.replace_name(new_name);
        };
        for segment_index in 0..block.segments.len() {
            apply_in_place_wrapped(SegmentIndex(segment_index), handle_name, &mut block);
        }

        Ok((block, true))
    }
}
