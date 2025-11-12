#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Skip {
    pub n: usize,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub remaining: DemultiplexedData<usize>,
}

impl Step for Skip {
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
            self.remaining.insert(tag, self.n);
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
        if self.remaining.len() == 1 {
            let remaining = self.remaining.get_mut(&DemultiplexTag::default()).unwrap();
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
            for output_tag in block.output_tags.as_ref().unwrap().iter() {
                let remaining = self.remaining.get_mut(output_tag).unwrap();
                keep.push(*remaining == 0);
                *remaining = remaining.saturating_sub(1);
            }
            super::super::apply_bool_filter(&mut block, &keep);
            Ok((block, true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
