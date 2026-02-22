#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::transformations::prelude::*;

use super::super::extend_seed;
use rand::Rng;

/// Sample reads by probability. Cheap.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct Sample {
    pub p: f64,
    pub seed: u64,

    #[tpd(skip)] // eserde compatibility
    #[schemars(skip)]
    rng: Arc<Mutex<Option<rand_chacha::ChaChaRng>>>,
}

impl VerifyIn<PartialConfig> for PartialSample {
    fn verify(&mut self, _parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.p.verify(|p| {
            if *p <= 0.0 || *p >= 1.0 {
                Err(ValidationFailure::new(
                    "p must be in (0.0..1.0). Set to a unit scale probability > 0 and < 1.",
                    None,
                ))
            } else {
                Ok(())
            }
        });
        Ok(())
    }
}

impl Step for Sample {
    fn must_see_all_tags(&self) -> bool {
        true
    }

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
        mut block: FastQBlocksCombined,
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

        let keep = (0..block.segments[0].entries.len())
            .map(|_| rng.random_bool(self.p))
            .collect::<Vec<_>>();
        block.apply_bool_filter(&keep);
        Ok((block, true))
    }
}
