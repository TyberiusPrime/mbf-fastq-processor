#![allow(clippy::unnecessary_wraps)]

use crate::transformations::prelude::*;
use std::sync::atomic::Ordering;

pub(crate) fn default_sample_stride() -> u64 {
    1000
}

/// Spot check the read names matching across segments
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateReadPairing {
    pub sample_stride: u64,

    #[schemars(skip)]
    #[tpd(skip, default)]
    processed_reads: std::sync::atomic::AtomicU64,
}

impl PartialValidateReadPairing {
    pub fn new(sample_stride: Option<u64>) -> Self {
        Self {
            sample_stride: TomlValue::new_ok(
                sample_stride.unwrap_or_else(default_sample_stride),
                0..0,
            ),
            processed_reads: Some(Default::default()),
        }
    }
}

impl VerifyIn<PartialConfig> for PartialValidateReadPairing {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.sample_stride.or_with(default_sample_stride);
        if let Some(input_config) = parent.input.as_ref() {
            if input_config.get_segment_order().len() < 2 {
                return Err(ValidationFailure::new(
                    "ValidateReadPairing requires at least two input segments",
                    Some("Check your [input] section or remove the step"),
                ));
            }
        }
        self.sample_stride.verify(|v|
            if *v == 0 {
                Err(
                    ValidationFailure::new(
                    "Must be > 0", Some("sample_stride = n means every n-th read is checked, so choose a number > 0")))
            } else {Ok(())}
        );
        Ok(())
    }
}

impl Default for ValidateReadPairing {
    fn default() -> Self {
        Self {
            processed_reads: 0.into(),
            sample_stride: default_sample_stride(),
        }
    }
}

impl Step for ValidateReadPairing {
    fn transmits_premature_termination(&self) -> bool {
        true
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        if block.segments.is_empty() {
            return Ok((block, true));
        }

        let segment_count = block.segments.len();
        let reads_in_block = block.segments[0].entries.len();
        assert!(self.sample_stride > 0);

        let offset = self
            .processed_reads
            .fetch_add(reads_in_block as u64, Ordering::Relaxed);

        for read_idx in 0..reads_in_block {
            let global_index = offset + read_idx as u64;
            if !global_index.is_multiple_of(self.sample_stride) {
                continue;
            }

            let reference = block.segments[0].get(read_idx);
            let reference_name = reference.name();

            if reference_name.is_empty() {
                bail!(
                    "ValidateReadPairing encountered an empty read name for the first segment, at sampled read index {global_index}"
                );
            }

            for segment_idx in 1..segment_count {
                let candidate = block.segments[segment_idx].get(read_idx);
                let candidate_name = candidate.name();

                if reference_name.len() != candidate_name.len() {
                    bail!(
                        "ValidateReadPairing detected mismatched read name lengths.
Occured near read {global_index} (0-based, sampled every {} reads).
First segment name: {:?}, length {},
other segment name: {:?}, length {}. \
Fix your input,
    or disable this sampling check by setting options.spot_check_read_pairing = false
    or add a ValidateName step to choose a custom read_name_end_char.",
                        self.sample_stride,
                        BStr::new(reference_name),
                        reference_name.len(),
                        BStr::new(candidate_name),
                        candidate_name.len(),
                    );
                } else {
                    let dist = bio::alignment::distance::hamming(reference_name, candidate_name);
                    if dist > 1 {
                        bail!("ValidateReadPairing detected mismatched read names near read {global_index}.
Had a hamming distance above 1: {dist}
First segment's read: {reference_name}
Mismatched read     : {candidate_name}
Fix your input,
    or disable sampling check by setting options.spot_check_read_pairing = false
    or add a ValidateName step to choose a custom read_name_end_char and non-hamming comparison.
",

                        reference_name = BStr::new(reference_name),
                        candidate_name = BStr::new(candidate_name),
                    );
                    }
                }
            }
        }
        Ok((block, true))
    }
}
