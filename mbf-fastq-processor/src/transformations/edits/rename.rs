#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::transformations::prelude::*;
use std::sync::atomic::Ordering;

use crate::config::deser::{bstring_from_string, u8_regex_from_string};
use bstr::{BString, ByteSlice};

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rename {
    #[serde(deserialize_with = "u8_regex_from_string")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,
    #[serde(deserialize_with = "bstring_from_string")]
    #[schemars(with = "String")]
    pub replacement: BString,
    #[serde(default)]
    #[serde(skip)]
    next_index: std::sync::atomic::AtomicU64,
}

impl Step for Rename {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let Some(first_segment) = block.segments.first() else {
            return Ok((block, true));
        };

        let read_count = first_segment.entries.len();
        if read_count == 0 {
            return Ok((block, true));
        }

        let replacement_bytes = self.replacement.as_bytes();
        let base_index = self
            .next_index
            .fetch_add(read_count as u64, Ordering::Relaxed);

        for segment_block in &mut block.segments {
            for read_idx in 0..read_count {
                let current_index = base_index.wrapping_add(read_idx as u64); //can overflow,
                //just like the atomic
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

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
