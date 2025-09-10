#![allow(clippy::unnecessary_wraps)] //eserde false positives

use super::super::{apply_bool_filter, extend_seed, Segment, Step};
use crate::demultiplex::Demultiplexed;
use rand::Rng;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    #[validate(minimum = 0.)]
    #[validate(maximum = 1.)]
    pub p: f32,
    pub seed: u64,
}

impl Step for Sample {
    fn apply(
        &mut self,
        mut block: crate::io::FastQBlocksCombined,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        use rand_chacha::rand_core::SeedableRng;
        let extended_seed = extend_seed(self.seed);

        // Singlecore approach to avoid reinitializing RNG
        let mut rng = rand_chacha::ChaChaRng::from_seed(extended_seed);
        let keep = (0..block.segments[0].entries.len())
            .map(|_| rng.random_bool(f64::from(self.p)))
            .collect::<Vec<_>>();
        apply_bool_filter(&mut block, &keep);
        (block, true)
    }
}
