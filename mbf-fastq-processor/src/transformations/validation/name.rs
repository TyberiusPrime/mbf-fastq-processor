#![allow(clippy::unnecessary_wraps)]
use std::sync::atomic::Ordering;

// eserde false positives
use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::transformations::{prelude::*, read_name_canonical_prefix_strict};

use crate::transformations::validation::read_pairing::default_sample_stride;

/// Validate that read names between segments match
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct ValidateName {
    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub readname_end_char: Option<u8>,

    pub sample_stride: u64,

    #[schemars(skip)]
    #[tpd(skip, default)]
    processed_reads: std::sync::atomic::AtomicU64,
}

impl VerifyIn<PartialConfig> for PartialValidateName {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        if let Some(input_config) = parent.input.as_ref() {
            if input_config.get_segment_order().len() < 2 {
                return Err(ValidationFailure::new(
                    "ValidateName requires at least two input segments",
                    Some("Check your [input] section or remove the step"),
                ));
            }
        }
        self.sample_stride.or_with(default_sample_stride);
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

impl Step for ValidateName {
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
                    "ValidateName encountered an empty read name for segment 0 at sampled read index {global_index}."
                );
            }

            let expected_prefix = read_name_canonical_prefix_strict(
                reference_name,
                self.readname_end_char,
            )
            .with_context(|| {
                format!(
                    "ValidateName did not find the expected readname_end_char '{:?}' in read {} at read index {global_index} on the first segment",
                    self.readname_end_char,
                    BStr::new(reference_name)
                )
            })?;

            for segment_idx in 1..segment_count {
                let candidate = block.segments[segment_idx].get(read_idx);
                let candidate_name = candidate.name();

                let candidate_prefix =
                    read_name_canonical_prefix_strict(candidate_name, self.readname_end_char)

            .with_context(|| {
                format!(
                    "ValidateName did not find the expected readname_end_char '{:?}' in read {} at read index {global_index} on another segment",
                    self.readname_end_char,
                    BStr::new(candidate_name)
                )
            })?;

                if candidate_prefix != expected_prefix {
                    bail!(
                        "ValidateNamew detected mismatched read names near read {global_index} (0-based, sampled every {} reads).
First segment (prefix): {},
Other segment (prefix): {}
First segment (full name): {},
Other segment (full name): {}
Fix your input, 
    or adjust the read_name_end_char option on this step, 
    or remove the ValidateName step.
                        ",
                        self.sample_stride,
                        BStr::new(expected_prefix),
                        BStr::new(candidate_prefix),
                        BStr::new(reference_name),
                        BStr::new(candidate_name),
                    );
                }
            }
        }

        Ok((block, true))
    }
}
