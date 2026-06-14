use crate::transformations::prelude::*;
use fastqrab_io::CompressionFormat;

use super::output_fastq::{declare_text_output, interleave_present};
use super::{RecordOutputState, collect_segment_list, verify_record_targets};

/// Write reads to FASTA file(s) as a pipeline step.
///
/// Like [`super::OutputFASTQ`] but emits FASTA records (no quality line).
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct OutputFASTA {
    /// Override the file suffix (defaults to `fasta`, plus the compression suffix).
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    suffix: Option<String>,

    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    compression: CompressionFormat,
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    compression_level: Option<u8>,
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    compression_threads: usize,

    /// Segments to write to individual files. Defaults to all input segments.
    #[tpd(default, alias = "segments")]
    output: Option<Vec<String>>,

    /// Segments to interleave into a single file (or stdout).
    #[tpd(default, alias = "interleaved")]
    interleave: Option<Vec<String>>,

    #[tpd(default)]
    stdout: bool,

    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    chunksize: Option<usize>,

    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    output_hash_uncompressed: bool,
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    output_hash_compressed: bool,

    #[tpd(skip, default)]
    #[schemars(skip)]
    output_state: Option<Arc<Mutex<RecordOutputState>>>,
}

const FORMAT: FileFormat = FileFormat::Fasta;

impl VerifyIn<PartialConfig> for PartialOutputFASTA {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.compression_threads.or(1);
        self.compression_threads.verify(|threads| {
            if *threads == 0 {
                Err(ValidationFailure::new(
                    "Must not be 0.",
                    Some("'compression_threads' must be greater than zero."),
                ))
            } else {
                Ok(())
            }
        });
        if let Some(Some(_)) = self.compression_level.value {
            crate::config::validate_compression_level_u8(
                &self.compression,
                &mut self.compression_level,
                &FORMAT,
            );
        }
        self.chunksize.verify(|chunk_size| {
            if let Some(chunk_size) = chunk_size.as_ref() {
                if *chunk_size == 0 {
                    return Err(ValidationFailure::new(
                        "Must not be 0.",
                        Some("'chunksize' must be greater than zero when specified."),
                    ));
                } else if let Some(true) = self.stdout.as_ref() {
                    return Err(ValidationFailure::new(
                        "Invalid when stdout = true",
                        Some("Either remove 'chunksize' or set 'stdout' to false"),
                    ));
                }
            }
            Ok(())
        });
        verify_record_targets(
            parent,
            &mut self.output,
            &mut self.interleave,
            &mut self.stdout,
            true,
        );
        Ok(())
    }
}

impl TagUser for PartialTaggedVariant<PartialOutputFASTA> {
    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let inner = self
            .toml_value
            .value
            .as_ref()
            .expect("declare_output_files called without successful verification");
        declare_text_output(
            FORMAT,
            inner.suffix.as_ref().and_then(|x| x.as_ref()),
            inner.compression.as_ref().copied().unwrap_or_default(),
            inner
                .compression_level
                .as_ref()
                .and_then(|x| x.as_ref())
                .copied(),
            inner.compression_threads.as_ref().copied().unwrap_or(1),
            *inner.output_hash_uncompressed.unwrap_ref(),
            *inner.output_hash_compressed.unwrap_ref(),
            &collect_segment_list(&inner.output),
            interleave_present(&inner.interleave).then(|| collect_segment_list(&inner.interleave)),
            *inner.stdout.unwrap_ref(),
            inner.chunksize.as_ref().and_then(|x| x.as_ref()).copied(),
            self.toml_value.span(),
        )
    }
}

impl Step for OutputFASTA {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        false
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        _demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let segments = self.output.clone().unwrap_or_default();
        let state = RecordOutputState::from_step_output_files(
            &mut output_files,
            &input_info.segment_order,
            &segments,
            self.interleave.as_deref(),
            self.stdout,
        );
        self.output_state = Some(Arc::new(Mutex::new(state)));
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        _input_info: &InputInfo,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<(FastQBlocksCombined, bool)> {
        self.output_state
            .as_ref()
            .expect("output_state should have been set in init")
            .lock()
            .expect("lock poisoned")
            .write_block(&block, demultiplex_info).context("Error in OutputFASTQ stage")?;
        Ok((block, true))
    }

    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        self.output_state
            .as_ref()
            .expect("output_state should have been set in init")
            .lock()
            .expect("lock poisoned")
            .finish()?;
        Ok(None)
    }
}
