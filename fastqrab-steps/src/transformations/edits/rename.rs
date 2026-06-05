use bstr::ByteSlice;

use crate::transformations::prelude::*;
use fastqrab_config::{tpd_adapt_bstring, tpd_adapt_regex};
use std::sync::atomic::Ordering;

/// Rename (and/or renumber) reads by applying a regex
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Rename {
    #[tpd(with = "tpd_adapt_regex")]
    #[schemars(with = "String")]
    pub search: regex::bytes::Regex,
    #[tpd(with = "tpd_adapt_bstring")]
    #[schemars(with = "String")]
    pub replacement: BString,

    #[schemars(skip)]
    #[tpd(skip)]
    next_index: std::sync::atomic::AtomicU64,

    #[schemars(skip)]
    #[tpd(skip)]
    needs_counting: bool,
}

impl VerifyIn<PartialConfig> for PartialRename {
    fn verify(
        &mut self,
        _parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        // Check if replacement contains the {{READ_INDEX}} placeholder
        self.needs_counting = Some(
            if let Some(replacement) = self.replacement.as_ref()
                && replacement.contains_str("{{READ_INDEX}}")
            {
                true
            } else {
                false
            },
        );
        self.next_index = Some(std::sync::atomic::AtomicU64::new(0));
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialRename> {}

impl Step for Rename {
    fn apply(
        &self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let first_segment = block.segments.first().expect("no segments!?");

        let read_count = first_segment.len();
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

        for segment in &mut block.segments {
            let mut new_names = StringPodBuilder::with_capacity(segment.names.get(0).len(), read_count);
            for (read_idx, name) in segment.names.iter().enumerate() {
                //just like the atomic
                let mut renamed = self
                    .search
                    .replace_all(name, replacement_bytes)
                    .into_owned();
                if self.needs_counting {
                    let current_index = base_index.wrapping_add(read_idx as u64); //can overflow,
                    renamed = renamed.replace(b"{{READ_INDEX}}", current_index.to_string());
                }
                new_names.push(&renamed);
            }
            segment.names = new_names.finish();
        }

        Ok((block, true))
    }

    #[mutants::skip] // replacing with constant true will obvs. work, this is an optimization.
    fn needs_serial(&self) -> bool {
        self.needs_counting
    }
}
