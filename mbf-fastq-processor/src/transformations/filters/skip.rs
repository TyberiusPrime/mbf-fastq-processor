#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

/// Skip the first n reads
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Skip {
    pub n: usize,

    #[tpd(skip)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[schemars(skip)]
    pub remaining: Arc<Mutex<DemultiplexedData<usize>>>,
}

impl Step for Skip {
    fn must_see_all_tags(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &std::path::Path,
        _output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut remaining = self.remaining.lock().expect("mutex poisoned");
        for tag in demultiplex_info.iter_tags() {
            remaining.insert(tag, self.n);
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
        let mut remaining = self.remaining.lock().expect("mutex poisoned");
        if remaining.len() == 1 {
            let remaining = remaining
                .get_mut(&DemultiplexTag::default())
                .expect("default tag must exist in remaining");
            if *remaining == 0 {
                Ok((block, true))
            } else if *remaining >= block.len() {
                *remaining -= block.len();
                Ok((block.empty(), true))
            } else {
                let here = (*remaining).min(block.len());
                *remaining -= here;
                block.drain(0..here);
                Ok((block, true))
            }
        } else {
            let mut keep = Vec::new();
            for output_tag in block
                .output_tags
                .as_ref()
                .expect("output_tags must be set when demultiplexing")
            {
                let remaining = remaining
                    .get_mut(output_tag)
                    .expect("output_tag must exist in remaining");
                keep.push(*remaining == 0);
                *remaining = remaining.saturating_sub(1);
            }
            block.apply_bool_filter(&keep);
            Ok((block, true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}