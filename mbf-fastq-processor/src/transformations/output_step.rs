use crate::config::{FileFormat, Output};
use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::transformations::prelude::*;
use anyhow::{Result, bail};
use std::io::Write;
use std::path::Path;

/// OutputStep allows writing the current pipeline state to files at any point in the pipeline.
/// This enables creating checkpoints or writing intermediate results.
#[derive(eserde::Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OutputStep {
    /// The output configuration for this step
    #[serde(flatten)]
    pub output_config: Output,

    #[serde(default)]
    #[serde(skip)]
    segment_writers: Vec<Option<HashedAndCompressedWriter<'static, ex::fs::File>>>,

    #[serde(default)]
    #[serde(skip)]
    segments_to_write: Vec<usize>,
}

impl Clone for OutputStep {
    fn clone(&self) -> Self {
        panic!("No cloning needs_serial steps (OutputStep)")
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for OutputStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputStep")
            .field("output_config", &self.output_config)
            .finish()
    }
}

impl Step for OutputStep {
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate compression level
        crate::config::validate_compression_level_u8(
            self.output_config.compression,
            self.output_config.compression_level,
        )?;

        // Only support Fastq and Fasta formats for now
        if !matches!(
            self.output_config.format,
            FileFormat::Fastq | FileFormat::Fasta
        ) {
            bail!(
                "OutputStep currently only supports 'Fastq' or 'Fasta' formats. Received: {:?}",
                self.output_config.format
            );
        }

        // Don't support stdout in OutputStep
        if self.output_config.stdout {
            bail!(
                "OutputStep does not support stdout output. Use the [output] section for stdout."
            );
        }

        // Don't support interleaving in OutputStep for now
        if self
            .output_config
            .interleave
            .as_ref()
            .is_some_and(|v| !v.is_empty())
        {
            bail!(
                "OutputStep does not support interleaving. Use the [output] section for interleaved output."
            );
        }

        // Don't support reports in OutputStep
        if self.output_config.report_html
            || self.output_config.report_json
            || self.output_config.report_timing
        {
            bail!(
                "OutputStep does not support reports. Reports should only be configured in the [output] section."
            );
        }

        // Don't support chunking in OutputStep
        if self.output_config.chunksize.is_some_and(|v| v > 0) {
            bail!(
                "OutputStep does not support chunking. Use the [output] section for chunked output."
            );
        }

        Ok(())
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        _output_prefix: &str,
        output_directory: &Path,
        output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        // Don't support demultiplexing in OutputStep yet
        if let OptDemultiplex::Yes(_) = demultiplex_info {
            bail!(
                "OutputStep does not yet support demultiplexing. Use the [output] section for demultiplexed output."
            );
        }

        let format_suffix = self.output_config.get_suffix();
        let prefix = &self.output_config.prefix;

        // Determine which segments to write
        let segments_to_output = self
            .output_config
            .output
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(&input_info.segment_order);

        // Map segment names to indices
        self.segments_to_write = Vec::new();
        for segment_name in segments_to_output {
            if let Some(idx) = input_info
                .segment_order
                .iter()
                .position(|s| s == segment_name)
            {
                self.segments_to_write.push(idx);
            } else {
                bail!(
                    "OutputStep: Segment '{}' not found in input segments",
                    segment_name
                );
            }
        }

        // Create writers for each segment
        self.segment_writers = (0..input_info.segment_order.len()).map(|_| None).collect();

        for &segment_idx in &self.segments_to_write {
            let segment_name = &input_info.segment_order[segment_idx];
            let base = crate::join_nonempty(
                [prefix.as_str(), segment_name.as_str()],
                output_ix_separator,
            );
            let full_path = output_directory.join(format!("{base}.{format_suffix}"));

            crate::output::ensure_output_destination_available(&full_path, allow_overwrite)?;

            let file = ex::fs::File::create(&full_path)?;
            let writer = HashedAndCompressedWriter::new(
                file,
                self.output_config.compression,
                self.output_config.output_hash_uncompressed,
                self.output_config.output_hash_compressed,
                self.output_config.compression_level,
                None, // no simulated failure
            )?;

            self.segment_writers[segment_idx] = Some(writer);
        }

        Ok(None)
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        // Write each segment's data to its corresponding file
        for &segment_idx in &self.segments_to_write {
            if let Some(writer) = &mut self.segment_writers[segment_idx] {
                let segment_block = &block.segments[segment_idx];

                // Write each read in the block
                let mut iter = segment_block.get_pseudo_iter();
                while let Some(read) = iter.pseudo_next() {
                    match self.output_config.format {
                        FileFormat::Fastq => {
                            writer.write_all(b"@")?;
                            writer.write_all(read.name())?;
                            writer.write_all(b"\n")?;
                            writer.write_all(read.seq())?;
                            writer.write_all(b"\n+\n")?;
                            writer.write_all(read.qual())?;
                            writer.write_all(b"\n")?;
                        }
                        FileFormat::Fasta => {
                            writer.write_all(b">")?;
                            writer.write_all(read.name())?;
                            writer.write_all(b"\n")?;
                            writer.write_all(read.seq())?;
                            writer.write_all(b"\n")?;
                        }
                        _ => unreachable!("validate_others ensures only Fastq/Fasta"),
                    }
                }
            }
        }

        // Pass through data unchanged
        Ok((block, true))
    }

    fn finalize(
        &mut self,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        // Close all writers
        for writer in self.segment_writers.iter_mut().filter_map(Option::take) {
            writer.finish();
        }
        Ok(None)
    }

    fn needs_serial(&self) -> bool {
        // Output steps need to be serial to maintain correct ordering
        true
    }
}
