use crate::transformations::prelude::*;
use fastqrab_io::CompressionFormat;
use fastqrab_io::io::output::chunked_writer::BamSinkOptions;

use super::output_fastq::interleave_present;
use super::{
    RecordOutputDeclSpec, RecordOutputState, build_record_declarations, collect_segment_list,
    sink_config, verify_record_targets,
};
use crate::config::{BamOutputOptions, PartialBamOutputOptions};

/// Write reads to BAM file(s) as a pipeline step.
///
/// Replicates the BAM behaviour of the legacy `[output]` section: per-segment
/// files, interleaved output, chunking and compressed hashing.
///
/// NOTE on scope: BAM auxiliary-tag export (`bam.tags` / `tag_to_bam_tag`),
/// reference assignment (`bam.tag_to_reference`) and merging of demultiplexed
/// BAMs (`bam.merge_demultiplexed`) are accepted and round-tripped in the
/// config, but their *application* is deferred to the pipeline-wiring follow-up.
/// Resolving references / building the BAM header happens when the writer is
/// created (at config-verify time, from the partial config) and additionally
/// requires barcode and runtime data that the step-output declaration path does
/// not yet receive; `merge` lives in the binary crate. The legacy `[output]`
/// path still performs all of these today.
#[derive(JsonSchema, Clone)]
#[tpd]
#[derive(Debug)]
pub struct OutputBAM {
    /// Override the file suffix (defaults to `bam`).
    #[tpd(default)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    suffix: Option<String>,

    /// BGZF compression level (0-9).
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

    /// Segments to interleave into a single file.
    #[tpd(default, alias = "interleaved")]
    interleave: Option<Vec<String>>,

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
    output_hash_compressed: bool,

    /// BAM-specific options (comment separator, tag exports, reference assignment).
    /// Carried for round-trip + future wiring; only `comment_separation_char` is
    /// applied today (see the type-level note).
    #[tpd(nested)]
    #[expect(
        dead_code,
        reason = "read in declare_output_files via the partial config"
    )]
    bam: Option<BamOutputOptions>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    output_state: Option<Arc<Mutex<RecordOutputState>>>,
}

const FORMAT: FileFormat = FileFormat::Bam;

impl VerifyIn<PartialConfig> for PartialOutputBAM {
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
            let compression = TomlValue::new_ok(CompressionFormat::Uncompressed, 0..0);
            crate::config::validate_compression_level_u8(
                &compression,
                &mut self.compression_level,
                &FORMAT,
            );
        }
        self.chunksize.verify(|chunk_size| {
            if let Some(chunk_size) = chunk_size.as_ref()
                && *chunk_size == 0
            {
                return Err(ValidationFailure::new(
                    "Must not be 0.",
                    Some("'chunksize' must be greater than zero when specified."),
                ));
            }
            Ok(())
        });
        // BAM cannot be written to stdout; pass a throwaway stdout flag.
        let mut stdout = TomlValue::new_ok(false, 0..0);
        verify_record_targets(
            parent,
            &mut self.output,
            &mut self.interleave,
            &mut stdout,
            false,
        );
        Ok(())
    }
}

/// Build the (currently minimal) BAM sink options carried in each declaration.
///
/// Only `comment_separation_char` is applied today; tag export / reference
/// resolution are deferred (see the type-level note).
fn declare_bam_options(comment_separation_char: u8) -> BamSinkOptions {
    let mut opts = BamSinkOptions {
        comment_separation_char,
        tag_to_bam_tags: Vec::new(),
        tag_to_reference: None,
        reference_sequences: Arc::new(Vec::new()),
        shared_header: None,
    };
    opts.build_shared_header();
    opts
}

impl TagUser for PartialTaggedVariant<PartialOutputBAM> {
    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let inner = self
            .toml_value
            .value
            .as_ref()
            .expect("declare_output_files called without successful verification");

        let comment_separation_char = inner
            .bam
            .as_ref()
            .and_then(|b| b.as_ref())
            .and_then(|b| b.comment_separation_char.as_ref().copied())
            .unwrap_or(b' ');

        let bam_options = declare_bam_options(comment_separation_char);

        let segments = collect_segment_list(&inner.output);
        let interleave =
            interleave_present(&inner.interleave).then(|| collect_segment_list(&inner.interleave));
        let suffix = FORMAT.get_suffix(
            CompressionFormat::Uncompressed,
            inner.suffix.as_ref().and_then(|x| x.as_ref()),
        );
        let spec = RecordOutputDeclSpec {
            format: FORMAT,
            suffix,
            segments: &segments,
            interleave: interleave.as_deref(),
            stdout: false,
            sink_config: sink_config(
                CompressionFormat::Uncompressed,
                inner
                    .compression_level
                    .as_ref()
                    .and_then(|x| x.as_ref())
                    .copied(),
                inner.compression_threads.as_ref().copied().unwrap_or(1),
                false,
                *inner.output_hash_compressed.unwrap_ref(),
            ),
            chunksize: inner.chunksize.as_ref().and_then(|x| x.as_ref()).copied(),
            bam_options: Some(bam_options),
            span: self.toml_value.span(),
        };
        build_record_declarations(&spec)
    }
}

impl Step for OutputBAM {
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
            false,
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
            .write_block(&block, demultiplex_info).context("Error in OutputBAM stage")?;
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
