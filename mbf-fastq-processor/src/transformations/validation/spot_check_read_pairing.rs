#![allow(clippy::unnecessary_wraps)]

use crate::config::deser::single_u8_from_string;
use crate::transformations::prelude::*;
use crate::transformations::read_name_canonical_prefix;
use anyhow::{Result, anyhow, bail};
use bstr::BStr;

fn default_sample_stride() -> usize {
    1000
}

#[derive(eserde::Deserialize, Debug, Clone, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SpotCheckReadPairing {
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    processed_reads: usize,

    #[serde(default = "default_sample_stride")]
    pub sample_stride: usize,

    #[serde(default, deserialize_with = "single_u8_from_string")]
    #[serde(alias = "read_name_end_char")]
    pub readname_end_char: Option<u8>,
}

impl Default for SpotCheckReadPairing {
    fn default() -> Self {
        Self {
            processed_reads: 0,
            sample_stride: default_sample_stride(),
            readname_end_char: Some(b'/'),
        }
    }
}

impl Step for SpotCheckReadPairing {
    fn transmits_premature_termination(&self) -> bool {
        true
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        if input_def.segment_count() <= 1 {
            bail!("SpotCheckReadPairing requires at least two input segments");
        }
        if self.sample_stride == 0 {
            bail!("Sample stride must be a positive integer");
        }
        Ok(())
    }

    fn apply(
        &mut self,
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

        for read_idx in 0..reads_in_block {
            let global_index = self.processed_reads + read_idx;
            if global_index % self.sample_stride != 0 {
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

        self.processed_reads += reads_in_block;

        if let Some(error) = error {
            Err(error)
        } else {
            Ok((block, true))
        }
    }
}
