#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::{Step};
use crate::{
    config::deser::{bstring_from_string, u8_regex_from_string},
    demultiplex::Demultiplexed,
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
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let handle_name = |read: &mut crate::io::WrappedFastQReadMut| {
            let name = read.name();
            let new_name = self
                .search
                .replace_all(name, self.replacement.as_bytes())
                .into_owned();
            read.replace_name(new_name);
        };
        todo!(); // we need to have the target names available, I guess? maybe copy them in
        // validate?
        /* apply_in_place_wrapped(Segment::Read1, handle_name, &mut block);
        if block.read2.is_some() {
            apply_in_place_wrapped(Segment::Read2, handle_name, &mut block);
        }
        if block.index1.is_some() {
            apply_in_place_wrapped(Segment::Index1, handle_name, &mut block);
        }
        if block.index2.is_some() {
            apply_in_place_wrapped(Segment::Index2, handle_name, &mut block);
        } */

        (block, true)
    }
}
