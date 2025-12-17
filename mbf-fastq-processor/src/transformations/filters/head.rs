#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Head {
    pub n: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub so_far: Arc<Mutex<DemultiplexedData<usize>>>,
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
        let mut so_far = self.so_far.lock().expect("lock poisoned");
        for tag in demultiplex_info.iter_tags() {
            so_far.insert(tag, 0);
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
        let mut so_far = self.so_far.lock().expect("lock poisoned");
        if so_far.len() == 1 {
            let so_far = so_far.get_mut(&0).expect("tag 0 must exist in so_far");
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
            for output_tag in block
                .output_tags
                .as_ref()
                .expect("output_tags must be set when demultiplexing")
            {
                let so_far = so_far
                    .get_mut(output_tag)
                    .expect("output_tag must exist in so_far");
                keep.push(*so_far < self.n);
                *so_far = so_far.saturating_add(1);
            }
            super::super::apply_bool_filter(&mut block, &keep);
            //we can stop the input if we have reached n in all values
            let stop = so_far.values().all(|&count| count >= self.n);
            Ok((block, !stop))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
