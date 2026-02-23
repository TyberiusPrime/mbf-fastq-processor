use super::prelude::*;
use rand::{Rng, SeedableRng};
use serde_json::json;
use std::thread;

/// A transformation that delays processing
/// by a random amount.
/// Used to inject chaos into test cases.
#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
#[allow(dead_code)]
pub struct _InternalDelay {
    ignored: Option<u8>, //tpd does not like empty structs
}

impl Step for Box<_InternalDelay> {
    fn apply(
        &self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let seed = block_no; //needs to be reproducible, but different for each block
        let seed_bytes = seed.to_le_bytes();

        // Extend the seed_bytes to 32 bytes
        let mut extended_seed = [0u8; 32];
        extended_seed[..8].copy_from_slice(&seed_bytes);
        let mut rng = rand_chacha::ChaCha20Rng::from_seed(extended_seed);

        let delay = rng.random_range(0..10);
        thread::sleep(std::time::Duration::from_millis(delay));
        Ok((block, true))
    }
}

/// An internal read counter, similar to `report::_ReportCount`
/// but it does not block premature termination.
/// We use this to test the head->early termination -> premature termination logic
#[tpd]
#[derive(Debug)]
pub struct _InternalReadCount {
    pub out_label: String,
    #[tpd(skip, default)]
    report_no: usize,
    #[tpd(skip, default)]
    count: std::sync::atomic::AtomicUsize,
}

impl _InternalReadCount {
    pub fn new(out_label: String, report_no: usize) -> Self {
        Self {
            out_label,
            report_no,
            count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

impl Step for Box<_InternalReadCount> {
    // can run in prallel, since it's atomic.

    // fn transmits_premature_termination(&self) -> bool {
    //     true // That's the magic as opposed to the usual reports
    //     but this is the default for steps.
    // }
    fn apply(
        &self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        self.count.fetch_add(
            block.segments[0].entries.len(),
            std::sync::atomic::Ordering::Relaxed,
        );
        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let mut contents = serde_json::Map::new();
        contents.insert("_InternalReadCount".to_string(), json!(self.count));

        Ok(Some(FinalizeReportResult {
            report_no: self.report_no,
            contents: serde_json::Value::Object(contents),
        }))
    }
}

/// An internal error inducer for testing
/// will make the *step* fail during processing.

#[derive(Clone, JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct _InduceFailure {
    msg: String,
}

impl Step for Box<_InduceFailure> {
    fn needs_serial(&self) -> bool {
        true
    }
    fn apply(
        &self,
        _block: crate::io::FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        bail!("Induced failure: {}", self.msg);
    }
}
