#![allow(clippy::unnecessary_wraps)] //eserde false positives
use crate::io::{FastQBlock, FastQElement, FastQRead};
use crate::transformations::prelude::*;
use serde_valid::Validate;

#[derive(eserde::Deserialize, Debug, Clone, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReservoirSample {
    #[validate(minimum = 1)]
    pub n: usize,
    pub seed: u64,
    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub buffer: Vec<Vec<FastQRead>>,
    #[serde(default)] // eserde compatibility
    #[serde(skip)]
    pub count: usize,
}

impl Step for ReservoirSample {
    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        use rand::Rng;
        use rand_chacha::rand_core::SeedableRng;

        if block.is_final {
            // First, accumulate any reads from the final block itself
            if self.buffer.is_empty() {
                self.buffer = vec![Vec::new(); block.segments.len()];
            }

            for (segment_idx, segment) in block.segments.iter().enumerate() {
                for read in &segment.entries {
                    // Clone the read to make it owned
                    let owned_read = FastQRead {
                        name: FastQElement::Owned(read.name.get(&segment.block).to_vec()),
                        seq: FastQElement::Owned(read.seq.get(&segment.block).to_vec()),
                        qual: FastQElement::Owned(read.qual.get(&segment.block).to_vec()),
                    };
                    self.buffer[segment_idx].push(owned_read);
                }
            }

            // Now perform reservoir sampling on all accumulated reads
            let extended_seed = super::super::extend_seed(self.seed);
            let mut rng = rand_chacha::ChaChaRng::from_seed(extended_seed);

            let segment_count = self.buffer.len();
            let mut output_blocks = Vec::new();

            for segment_idx in 0..segment_count {
                // Get all reads for this segment
                let all_reads = &self.buffer[segment_idx];

                // Perform reservoir sampling using Algorithm L
                let sample_size = self.n.min(all_reads.len());

                if sample_size > 0 {
                    // Initialize reservoir with first n items
                    let mut reservoir: Vec<usize> = (0..sample_size).collect();

                    // Process remaining items
                    for i in sample_size..all_reads.len() {
                        let j = rng.random_range(0..=i);
                        if j < sample_size {
                            reservoir[j] = i;
                        }
                    }

                    // Sort indices to maintain read order
                    reservoir.sort_unstable();

                    // Create output reads
                    let output_reads: Vec<FastQRead> = reservoir
                        .iter()
                        .map(|&idx| all_reads[idx].clone())
                        .collect();

                    output_blocks.push(FastQBlock {
                        block: Vec::new(),
                        entries: output_reads,
                    });
                } else {
                    output_blocks.push(FastQBlock::empty());
                }
            }

            let output = FastQBlocksCombined {
                segments: output_blocks,
                output_tags: block.output_tags.clone(),
                tags: None,
                is_final: true,
            };

            Ok((output, true))
        } else {
            // Accumulate reads from this block
            if self.buffer.is_empty() {
                self.buffer = vec![Vec::new(); block.segments.len()];
            }

            for (segment_idx, segment) in block.segments.iter().enumerate() {
                for read in &segment.entries {
                    // Clone the read to make it owned
                    let owned_read = FastQRead {
                        name: FastQElement::Owned(read.name.get(&segment.block).to_vec()),
                        seq: FastQElement::Owned(read.seq.get(&segment.block).to_vec()),
                        qual: FastQElement::Owned(read.qual.get(&segment.block).to_vec()),
                    };
                    self.buffer[segment_idx].push(owned_read);
                }
            }

            self.count += block.len();

            // Return empty block to continue processing
            Ok((block.empty(), true))
        }
    }

    fn needs_serial(&self) -> bool {
        true
    }
}
