use crate::transformations::prelude::*;
use crate::config::Output;
use anyhow::Result;

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
    is_initialized: bool,
}

impl Clone for OutputStep {
    fn clone(&self) -> Self {
        Self {
            output_config: self.output_config.clone(),
            is_initialized: false,
        }
    }
}

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
        // TODO: Validate output configuration similar to check_output in config.rs
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        _output_prefix: &str,
        _output_directory: &std::path::Path,
        _output_ix_separator: &str,
        _demultiplex_info: &OptDemultiplex,
        _allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        self.is_initialized = true;
        // TODO: Open output file handles
        Ok(None)
    }

    fn apply(
        &mut self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        // Pass through data unchanged
        // TODO: Write block to output files
        Ok((block, true))
    }

    fn needs_serial(&self) -> bool {
        // Output steps need to be serial to maintain correct ordering
        true
    }
}
