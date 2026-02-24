use crate::config::{CompressionFormat, FileFormat};
use crate::io::output::compressed_output::HashedAndCompressedWriter;
use crate::transformations::prelude::*;
use std::io::Write;

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>, DemultiplexTag);

struct DebugFile(ex::fs::File);

impl std::fmt::Debug for DebugFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ex::fs::File(...)")
    }
}

/// Inspect reads within the workflow
#[derive(JsonSchema)]
#[tpd]
pub struct Inspect {
    pub n: usize,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,

    pub infix: String,
    #[tpd(default)]
    pub suffix: Option<String>,
    #[tpd(default)]
    pub format: FileFormat,
    #[tpd(default)]
    pub compression: CompressionFormat,
    #[tpd(default)]
    pub compression_level: Option<u8>,

    #[tpd(skip)]
    #[schemars(skip)]
    pub collector: Arc<Mutex<Vec<Vec<NameSeqQualTuple>>>>,
    #[tpd(skip)]
    collected: std::sync::atomic::AtomicUsize,

    #[tpd(skip, default)]
    #[schemars(skip)]
    //we write either interleaved (one file) or one segment (one file)
    writer: Arc<Mutex<Option<DebugFile>>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    demultiplex_names: Option<DemultiplexedData<String>>,
}

impl VerifyIn<PartialConfig> for PartialInspect {
    fn verify(&mut self, parent: &PartialConfig) -> std::result::Result<(), ValidationFailure>
    where
        Self: Sized + toml_pretty_deser::Visitor,
    {
        self.segment.validate_segment(parent);
        self.format.verify(|format| {
            if !matches!(format, FileFormat::Fastq | FileFormat::Fasta) {
                return Err(ValidationFailure::new(
                    "Inspect step supports only 'fastq' or 'fasta' formats",
                    None,
                ));
            }
            Ok(())
        });
        crate::config::validate_compression_level_u8(
            &self.compression,
            &mut self.compression_level,
        );
        if let Some(MustAdapt::PostVerify(segment)) = self.segment.as_ref()
            && let Some(segment_order) = parent
                .input
                .as_ref()
                .map(crate::config::PartialInput::get_segment_order)
        {
            let n = self.n.as_ref().map_or(0, |x| *x);
            self.collector = Some(Arc::new(Mutex::new(match segment {
                SegmentIndexOrAll::All => (0..segment_order.len())
                    .map(|_| Vec::with_capacity(n))
                    .collect(),
                SegmentIndexOrAll::Indexed(_) => vec![Vec::with_capacity(n)],
            })));
        }
        self.collected = Some(std::sync::atomic::AtomicUsize::new(0));
        Ok(())
    }
}

impl Clone for Inspect {
    fn clone(&self) -> Self {
        panic!("No cloning needs_serial steps")
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl std::fmt::Debug for Inspect {
    #[mutants::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inspect")
            .field("n", &self.n)
            .field("segment", &self.segment)
            //        .field("segment_index", &self.segment_index)
            .field("infix", &self.infix)
            .field("suffix", &self.suffix)
            .field("format", &self.format)
            .field("compression", &self.compression)
            //       .field("compression_level", &self.compression_level)
            //.field("collected", &self.collected)
            .finish()
    }
}

impl Step for Inspect {
    fn needs_serial(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        input_info: &InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        output_ix_separator: &str,
        demultiplex_info: &OptDemultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let format_suffix = FileFormat::Fastq.get_suffix(self.compression, self.suffix.as_ref());

        let target = match self.segment {
            SegmentIndexOrAll::Indexed(idx) => input_info.segment_order[idx].clone(),
            SegmentIndexOrAll::All => "interleaved".to_string(),
        };

        let base = crate::join_nonempty(
            [output_prefix, self.infix.as_str(), target.as_str()],
            output_ix_separator,
        );

        let full_path = output_directory.join(format!("{base}.{format_suffix}"));
        crate::output::ensure_output_destination_available(&full_path, allow_overwrite)?;

        let report_file = ex::fs::File::create(full_path)?;
        self.writer = Arc::new(Mutex::new(Some(DebugFile(report_file))));

        if let OptDemultiplex::Yes(info) = demultiplex_info {
            self.demultiplex_names = Some(
                info.tag_to_name
                    .iter()
                    .filter_map(|(tag, name)| name.as_ref().map(|name| (*tag, name.clone())))
                    .collect(),
            );
        }
        Ok(None)
    }

    fn apply(
        &self,
        block: FastQBlocksCombined,
        input_info: &InputInfo,
        _block_no: usize,
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut collected = self.collected.load(std::sync::atomic::Ordering::Relaxed);
        if collected >= self.n {
            return Ok((block, true));
        }

        let mut collector = self.collector.lock().expect("collector mutex poisoned");
        let mut iter = block.get_pseudo_iter_including_tag();
        while let Some((read, tag)) = iter.pseudo_next() {
            if collected >= self.n {
                break;
            }

            match self.segment {
                SegmentIndexOrAll::All => {
                    for (collector_idx, segment_index) in
                        (0..input_info.segment_order.len()).enumerate()
                    {
                        let segment_read = &read.segments[segment_index];
                        collector[collector_idx].push((
                            segment_read.name().to_vec(),
                            segment_read.seq().to_vec(),
                            segment_read.qual().to_vec(),
                            tag,
                        ));
                    }
                }
                SegmentIndexOrAll::Indexed(idx) => {
                    let segment_read = &read.segments[idx];
                    collector[0].push((
                        segment_read.name().to_vec(),
                        segment_read.seq().to_vec(),
                        segment_read.qual().to_vec(),
                        tag,
                    ));
                }
            }

            collected += 1; //count per molecule, not per segment
        }
        self.collected
            .store(collected, std::sync::atomic::Ordering::Relaxed);
        Ok((block, true))
    }
    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let collector = self.collector.lock().expect("collector mutex poisoned");
        let collected = self.collected.load(std::sync::atomic::Ordering::Relaxed);
        // Build filename with format-specific suffix
        let report_file_handle = self
            .writer
            .lock()
            .expect("writer mutex poisoned")
            .take()
            .expect("writer must be set during initialization");

        let mut writer = HashedAndCompressedWriter::new(
            report_file_handle.0,
            self.compression,
            false, // hash_uncompressed
            false, // hash_compressed
            self.compression_level,
            None, // compression_threads
            None,
        )?;
        if !collector.is_empty() {
            let reads_to_write = collected.min(self.n);
            for read_idx in 0..reads_to_write {
                for segment_reads in collector.iter() {
                    if let Some((name, seq, qual, tag)) = segment_reads.get(read_idx) {
                        writer.write_all(b"@")?;
                        writer.write_all(name)?;
                        if let Some(demux_names) = &self.demultiplex_names
                            && let Some(demux_name) = demux_names.get(tag)
                        {
                            writer.write_all(b" Demultiplex=")?;
                            writer.write_all(demux_name.as_bytes())?;
                        }

                        writer.write_all(b"\n")?;
                        writer.write_all(seq)?;
                        writer.write_all(b"\n+\n")?;
                        writer.write_all(qual)?;
                        writer.write_all(b"\n")?;
                    }
                }
            }
        }

        writer.finish();
        Ok(None)
    }
}
