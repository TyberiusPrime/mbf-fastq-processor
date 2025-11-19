#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Head {
    /// Number of reads to keep
    pub n: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub so_far: DemultiplexedData<usize>,
}

impl Step for Head {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &std::path::Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        for tag in demultiplex_info.iter_tags() {
            self.so_far.insert(tag, 0);
        }
        Ok(None)
    }
    fn apply(
        &mut self,
        mut block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        if self.so_far.len() == 1 {
            let so_far = self.so_far.get_mut(&0).unwrap();
            if *so_far >= self.n {
                let empty = block.empty();
                Ok((empty, false))
            } else {
                //we know so_far is smaller than n
                let remaining = self.n.saturating_sub(*so_far);
                assert!(remaining > 0);
                block.resize(remaining.min(block.len()));
                let do_continue = remaining > block.len();
                *so_far += block.len();
                Ok((block, do_continue))
            }
        } else {
            let mut keep = Vec::new();
            for output_tag in block.output_tags.as_ref().unwrap() {
                let so_far = self.so_far.get_mut(output_tag).unwrap();
                keep.push(*so_far < self.n);
                *so_far = so_far.saturating_add(1);
            }
            super::super::apply_bool_filter(&mut block, &keep);
            //we can stop the input if we have reached n in all values
            let stop = self.so_far.values().all(|&count| count >= self.n);
            Ok((block, !stop))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
