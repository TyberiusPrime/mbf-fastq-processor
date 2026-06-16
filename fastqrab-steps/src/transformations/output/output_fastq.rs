use crate::transformations::prelude::*;
use fastqrab_io::CompressionFormat;

use super::{
    RecordOutputDeclSpec, RecordOutputState, build_record_declarations, collect_segment_list,
    sink_config, verify_chunk_size, verify_record_targets, verify_suffix,
    validate_compression_level_u8,
};

/// Write reads to FASTQ file(s) as a pipeline step.
///
/// Replicates the FASTQ behaviour of the legacy `[output]` section: per-segment
/// files, interleaved output, stdout, chunking, hashing and compression.
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct OutputFASTQ {
    /// Override the file suffix (defaults to `fq`, plus the compression suffix).
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
    pub compression: CompressionFormat,
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

const FORMAT: FileFormat = FileFormat::Fastq;

impl VerifyIn<PartialConfig> for PartialOutputFASTQ {
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
            validate_compression_level_u8(
                &self.compression,
                &mut self.compression_level,
            );
        }
        self.chunksize
            .verify(|chunk_size| verify_chunk_size(chunk_size, &self.stdout));
        self.suffix.verify(verify_suffix);
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

impl TagUser for PartialTaggedVariant<PartialOutputFASTQ> {
    fn declare_output_files(&self) -> Option<Vec<OutputDeclaration>> {
        if let Some(inner) = self.toml_value.as_ref() {
            Some(declare_text_output(
                FORMAT,
                inner.suffix.as_ref().and_then(|x| x.as_ref()),
                inner.compression.as_ref().copied().unwrap_or_default(),
                inner
                    .compression_level
                    .as_ref()
                    .and_then(|x| x.as_ref())
                    .copied(),
                *inner
                    .output_hash_uncompressed
                    .as_ref()
                    .expect("parent was ok"),
                *inner
                    .output_hash_compressed
                    .as_ref()
                    .expect("parent was ok"),
                &collect_segment_list(&inner.output),
                interleave_present(&inner.interleave)
                    .then(|| collect_segment_list(&inner.interleave)),
                *inner.stdout.as_ref().expect("parent was ok"),
                inner.chunksize.as_ref().and_then(|x| x.as_ref()).copied(),
                self.toml_value.span(),
            ))
        } else {
            Some(vec![]) //there should be output files, but we can't name them.
        }
    }
}

impl Step for OutputFASTQ {
    fn needs_serial(&self) -> bool {
        true
    }
    fn transmits_premature_termination(&self) -> bool {
        false // write all the reads even if a later Head stops the stream
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
            .write_block(&block, demultiplex_info)
            .context("Error in OutputFASTA stage")?;
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

/// True when an interleave list was actually provided/defaulted (Some(Some(_))).
pub(super) fn interleave_present(interleave: &super::SegmentList) -> bool {
    matches!(interleave.as_ref(), Some(Some(_)))
}

/// Shared declaration builder for the text (FASTQ/FASTA) output steps.
#[expect(clippy::too_many_arguments, reason = "all needed to build the spec")]
pub(super) fn declare_text_output(
    format: FileFormat,
    suffix_override: Option<&String>,
    compression: CompressionFormat,
    compression_level: Option<u8>,
    hash_uncompressed: bool,
    hash_compressed: bool,
    segments: &[String],
    interleave: Option<Vec<String>>,
    stdout: bool,
    chunksize: Option<usize>,
    span: std::ops::Range<usize>,
) -> Vec<OutputDeclaration> {
    let spec = RecordOutputDeclSpec {
        format,
        suffix: format.get_suffix(compression, suffix_override),
        segments,
        interleave: interleave.as_deref(),
        stdout,
        sink_config: sink_config(
            compression,
            compression_level,
            hash_uncompressed,
            hash_compressed,
        ),
        chunksize,
        bam_options: None,
        span,
    };
    build_record_declarations(&spec)
}
