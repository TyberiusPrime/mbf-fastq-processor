#![allow(clippy::unnecessary_wraps)]
//eserde false positives
use crate::io::FastQRead;
use crate::transformations::{extend_seed, prelude::*};
use rand::Rng;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReservoirSample {
    #[validate(minimum = 1)]
    pub n: usize,
    pub seed: u64,
    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub buffers: DemultiplexedData<Vec<Vec<FastQRead>>>,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub counts: DemultiplexedData<usize>,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    rng: Option<rand_chacha::ChaChaRng>,
}

impl Step for ReservoirSample {
    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &std::path::Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> anyhow::Result<Option<DemultiplexBarcodes>> {
        use rand_chacha::rand_core::SeedableRng;
        let extended_seed = extend_seed(self.seed);
        self.rng = Some(rand_chacha::ChaChaRng::from_seed(extended_seed));
        Ok(None)
    }
    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let rng = self.rng.as_mut().unwrap();
        let mut pseudo_iter = block.get_pseudo_iter_including_tag();
        while let Some((read, demultiplex_tag)) = pseudo_iter.pseudo_next() {
            let out = self
                .buffers
                .entry(demultiplex_tag)
                .or_default();
            let i = self.counts.entry(demultiplex_tag).or_insert(0);
            *i += 1;
            if out.len() < self.n {
                out.push(read.owned());
            } else {
                //algorithm R
                let j = rng.random_range(1..=*i);
                if j <= self.n {
                    out[j - 1] = read.owned();
                }
            }
        }
        if block.is_final {
            let mut output = block.empty();
            let buffers = std::mem::replace(&mut self.buffers, DemultiplexedData::new());
            for (demultiplex_tag, reads) in buffers {
                if let Some(demultiplex_tags) = output.output_tags.as_mut() {
                    for _ in 0..reads.len() {
                        demultiplex_tags.push(demultiplex_tag);
                    }
                }
                for molecule in reads {
                    for (segment_no, read) in molecule.into_iter().enumerate() {
                        output.segments[segment_no].entries.push(read);
                    }
                }
            }
            Ok((output, true))
        } else {
            // Return empty block to continue processing
            Ok((block.empty(), true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
