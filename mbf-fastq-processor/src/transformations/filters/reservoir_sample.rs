#![allow(clippy::unnecessary_wraps)]
//eserde false positives
use crate::io::FastQBlock;
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
    pub buffers: Arc<Mutex<DemultiplexedData<Vec<FastQBlock>>>>,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub counts: Arc<Mutex<DemultiplexedData<usize>>>,

    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    rng: Arc<Mutex<Option<rand_chacha::ChaChaRng>>>,
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
        self.rng = Arc::new(Mutex::new(Some(rand_chacha::ChaChaRng::from_seed(
            extended_seed,
        ))));
        Ok(None)
    }
    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut rng_lock = self.rng.lock();
        let rng = rng_lock
            .as_mut()
            .expect("rng mutex poisoned")
            .as_mut()
            .expect("rng must be initialized before process()");
        let mut pseudo_iter = block.get_pseudo_iter_including_tag();

        let mut buffer_lock = self.buffers.lock();
        let buffers = buffer_lock.as_mut().expect("buffers mutex poisoned");

        let mut counts_lock = self.counts.lock();
        let counts = counts_lock.as_mut().expect("counts mutex poisoned");

        while let Some((molecule, demultiplex_tag)) = pseudo_iter.pseudo_next() {
            let out = buffers.entry(demultiplex_tag).or_default();
            let i = counts.entry(demultiplex_tag).or_insert(0);
            *i += 1;

            if out.is_empty() || out[0].len() < self.n {
                for (segment_no, read) in molecule.segments.iter().enumerate() {
                    if out.len() <= segment_no {
                        out.push(FastQBlock::empty());
                    }
                    out[segment_no].append_read(read);
                }
            } else {
                //algorithm R
                let j = rng.random_range(1..=*i);
                if j <= self.n {
                    for (ii, read) in molecule.segments.iter().enumerate() {
                        out[ii].replace_read(j - 1, read);
                    }
                }
            }
        }

        if block.is_final {
            //we gotta copy it all back together, so no easy just hand out our internal
            //storage, I suppose.
            let mut output = block.empty();
            let buffers = buffers.replace(DemultiplexedData::new());
            for (demultiplex_tag, reads) in buffers {
                if let Some(demultiplex_tags) = output.output_tags.as_mut() {
                    for _ in 0..reads[0].len() {
                        demultiplex_tags.push(demultiplex_tag);
                    }
                }
                for (segment_no, molecule) in reads.iter().enumerate() {
                    for read_idx in 0..molecule.entries.len() {
                        output.segments[segment_no].append_read(&molecule.get(read_idx));
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
