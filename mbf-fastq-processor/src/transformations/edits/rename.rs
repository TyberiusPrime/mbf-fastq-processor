#![allow(clippy::unnecessary_wraps)] //eserde false positives

use crate::{config::deser::{tpd_adapt_bstring, tpd_adapt_regex}, transformations::prelude::*};
use std::sync::atomic::Ordering;

use bstr::{ByteSlice};

/// Rename (and/or renumber) reads by applying a regex
#[derive( JsonSchema)]
#[tpd]
#[derive( Debug)]
pub struct Rename {
    #[tpd(with="tpd_adapt_regex")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,
    #[tpd(with="tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub replacement: BString,

    #[schemars(skip)]
    #[tpd(skip)]
    next_index: std::sync::atomic::AtomicU64,

    #[schemars(skip)]
    #[tpd(skip)]
    needs_counting: bool,
}

impl Step for Rename {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Check if replacement contains the {{READ_INDEX}} placeholder
        if self.replacement.contains_str(b"{{READ_INDEX}}") {
            self.needs_counting = true;
        }
        Ok(None)
    }
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
        let base_index = if self.needs_counting {
            self.next_index
                .fetch_add(read_count as u64, Ordering::Relaxed)
        } else {
            0
        };

        for segment_block in &mut block.segments {
            for read_idx in 0..read_count {
                //just like the atomic
                let mut read = segment_block.get_mut(read_idx);
                let name = read.name();
                let renamed = self
                    .search
                    .replace_all(name, replacement_bytes)
                    .into_owned();
                if self.needs_counting {
                    let current_index = base_index.wrapping_add(read_idx as u64); //can overflow,
                    let renamed = renamed.replace(b"{{READ_INDEX}}", current_index.to_string());
                    read.replace_name(&renamed);
                } else {
                    read.replace_name(&renamed);
                }
            }
        }

        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        self.needs_counting
    }
}