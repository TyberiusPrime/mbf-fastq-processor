#![allow(clippy::unnecessary_wraps)]

use crate::config::deser::tpd_adapt_u8_from_byte_or_char;
use crate::transformations::prelude::*;
use crate::transformations::read_name_canonical_prefix;
use std::sync::atomic::Ordering;

fn default_sample_stride() -> u64 {
    1000
}

/// Spot check the read names matching across segments
#[derive(JsonSchema)]
#[tpd]
#[derive(Debug)]
pub struct SpotCheckReadPairing {
    pub sample_stride: u64,

    #[tpd(with = "tpd_adapt_u8_from_byte_or_char")]
    pub readname_end_char: Option<u8>,

    #[schemars(skip)]
    #[tpd(skip, default)]
    processed_reads: std::sync::atomic::AtomicU64,
}

impl VerifyIn<PartialConfig> for PartialSpotCheckReadPairing {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.sample_stride.or_with(default_sample_stride);
        if let Some(input_config) = parent.input.as_ref() {
            if input_config.get_segment_order().len() < 2 {
                return Err(ValidationFailure::new(
                    "SpotCheckReadPairing requires at least two input segments",
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

impl Default for SpotCheckReadPairing {
    fn default() -> Self {
        Self {
            processed_reads: 0.into(),
            sample_stride: default_sample_stride(),
            readname_end_char: Some(b'/'),
        }
    }
}

impl Step for SpotCheckReadPairing {
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

        let mut error = None;
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
                error = Some(anyhow!(
                    "SpotCheckReadPairing encountered an empty read name for segment 0 at sampled read index {global_index}"
                ));
                break;
            }

            let expected_prefix =
                read_name_canonical_prefix(reference_name, self.readname_end_char);

            for segment_idx in 1..segment_count {
                let candidate = block.segments[segment_idx].get(read_idx);
                let candidate_name = candidate.name();
                // if candidate.seq().iter().any(|x| *x == b'\r') {
                //     error = Some(anyhow!("Found a windows newline"));
                //     break;
                // }

                let candidate_prefix =
                    read_name_canonical_prefix(candidate_name, self.readname_end_char);

                if candidate_prefix != expected_prefix {
                    error = Some(anyhow!(
                        "SpotCheckReadPairing detected mismatched read names near read {global_index} (0-based, sampled every {} reads). Expected prefix {:?} from segment 0 name {:?}, but segment {segment_idx} provided prefix {:?} from name {:?}. Fix your input, or disable this sampling check by setting options.spot_check_read_pairing = false or add a ValidateName step to choose a custom read_name_end_char.",
                        self.sample_stride,
                        BStr::new(expected_prefix),
                        BStr::new(reference_name),
                        BStr::new(candidate_prefix),
                        BStr::new(candidate_name),
                    ));
                    break;
                }
            }

            if error.is_some() {
                break;
            }
        }

        if let Some(error) = error {
            Err(error)
        } else {
            Ok((block, true))
        }
    }
}
