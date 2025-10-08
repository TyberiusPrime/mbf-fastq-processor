#![allow(clippy::unnecessary_wraps)] //eserde false positives
use super::super::Step;
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
    #[serde(default)]
    #[serde(skip)]
    next_index: usize,
}

impl Step for Rename {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let Some(first_segment) = block.segments.first() else {
            return Ok((block, true));
        };

        let read_count = first_segment.entries.len();
        if read_count == 0 {
            return Ok((block, true));
        }

        let replacement_bytes = self.replacement.as_bytes();
        let base_index = self.next_index;
        let next_index_after_block = base_index
            .checked_add(read_count)
            .expect("Rename read index overflowed at usize::MAX");

        for segment_block in &mut block.segments {
            for read_idx in 0..read_count {
                let current_index = base_index + read_idx; //cant overflow, checked above
                let mut read = segment_block.get_mut(read_idx);
                let name = read.name();
                let renamed = self
                    .search
                    .replace_all(name, replacement_bytes)
                    .into_owned();
                let renamed = renamed.replace(b"{{READ_INDEX}}", current_index.to_string());
                read.replace_name(renamed);
            }
        }

        self.next_index = next_index_after_block;

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
