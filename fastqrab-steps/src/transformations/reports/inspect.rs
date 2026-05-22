use std::cell::RefCell;
use std::rc::Rc;

use crate::transformations::prelude::*;
use fastqrab_io::io::WrappedFastQRead;
use fastqrab_io::{CompressionFormat, FileFormat};

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>, DemultiplexTag);

/// Inspect reads within the workflow
#[derive(JsonSchema)]
#[tpd]
pub struct Inspect {
    pub n: usize,

    #[schemars(with = "String")]
    #[tpd(adapt_in_verify(String))]
    segment: SegmentIndexOrAll,

    pub infix: String,
    #[tpd(skip, default)]
    #[schemars(skip)]
    #[allow(dead_code)]
    resolved_segment_name: String,
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
    writer: Arc<Mutex<Option<ChunkedRecordWriter>>>,

    #[tpd(skip, default)]
    #[schemars(skip)]
    demultiplex_names: Option<DemultiplexedData<String>>,
}

impl VerifyIn<PartialConfig> for PartialInspect {
    fn verify(
        &mut self,
        parent: &PartialConfig,
        _options: &VerifyOptions,
    ) -> std::result::Result<(), ValidationFailure>
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
            self.format.as_ref().unwrap_or(&FileFormat::Fastq),
        );
        if let Some(MustAdapt::PostVerify(segment)) = self.segment.as_ref()
            && let Some(segment_order) = parent
                .input
                .as_ref()
                .map(crate::config::PartialInput::get_segment_order)
        {
            let n = self.n.as_ref().map_or(0, |x| *x);
            let target_name = match segment {
                SegmentIndexOrAll::All => "interleaved".to_string(),
                SegmentIndexOrAll::Indexed(idx) => segment_order
                    .get(idx.as_index())
                    .cloned()
                    .unwrap_or_default(),
            };
            self.resolved_segment_name = Some(target_name);
            self.collector = Some(Arc::new(Mutex::new(match segment {
                SegmentIndexOrAll::All => (0..segment_order.len())
                    .map(|_| Vec::with_capacity(n))
                    .collect(),
                SegmentIndexOrAll::Indexed(_) => vec![Vec::with_capacity(n)],
            })));
        } // cov:excl-line
        self.collected = Some(std::sync::atomic::AtomicUsize::new(0));
        Ok(())
    }
}

// cov:excl-start
#[expect(
    clippy::missing_fields_in_debug,
    reason = "that's why we have a manual Debug impl"
)]
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
// cov:excl-stop

impl TagUser for PartialTaggedVariant<PartialInspect> {
    fn declare_output_files(&self) -> Vec<OutputDeclaration> {
        let inner = self
            .toml_value
            .value
            .as_ref()
            .expect("can't call declare_output_files when validation failed");
        let infix = inner.infix.as_ref().cloned().unwrap_or_default();
        let segment_name = inner
            .resolved_segment_name
            .as_deref()
            .unwrap_or_default()
            .to_string();
        let mut infix_parts = vec![infix];
        if !segment_name.is_empty() {
            infix_parts.push(segment_name);
        }
        let compression = inner.compression.as_ref().copied().unwrap_or_default();
        let compression_level = inner
            .compression_level
            .as_ref()
            .and_then(|x| x.as_ref())
            .copied();
        let format = inner.format.as_ref().copied().unwrap_or_default();
        let custom_suffix = inner.suffix.as_ref().and_then(|opt| opt.as_ref());
        let suffix = format.get_suffix(compression, custom_suffix);
        vec![OutputDeclaration {
            id: "inspect".to_string(),
            target: WriteTargetConfig::new(infix_parts, suffix),
            sink_config: SinkConfig {
                compression,
                compression_level,
                compression_threads: None,
                hash_uncompressed: false,
                hash_compressed: false,
                simulated_failure: None,
            },
            format,
            chunk_policy: ChunkPolicy::default(),
            bam_options: None,
            singleton: true,
            span: inner.infix.span(),
        }]
    }

    fn get_tag_usage(
        &mut self,
        tags_available: &IndexMap<TagLabel, TagMetadata>,
        _segment_order: &[String],
    ) -> Option<TagUsageInfo<'_>> {
        let final_in_labels: Vec<_> = tags_available.keys().cloned().collect();
        let toml_source = Rc::new(RefCell::new((
            &mut self.toml_value.state,
            &mut self.toml_value.help,
        )));
        let used_tags = final_in_labels
            .into_iter()
            .map(|tag| {
                Some(UsedTag {
                    name: tag,
                    accepted_tag_types: ANY_TAG_TYPE,
                    toml_source: toml_source.clone(),
                    further_help: None,
                })
            })
            .collect();
        Some(TagUsageInfo {
            used_tags,
            must_see_all_tags: true,
            ..Default::default()
        })
    }
}

impl Step for Inspect {
    #[mutants::skip] // doesn't always trigger if replaced by false, but often enough
    fn needs_serial(&self) -> bool {
        true
    }

    fn init(
        &mut self,
        _input_info: &InputInfo,
        mut output_files: StepOutputFiles,
        demultiplex_info: &OptDemultiplex,
    ) -> Result<Option<DemultiplexBarcodes>> {
        let mut per_tag = output_files.take("inspect");
        let writer = per_tag
            .remove(&0)
            .expect("tag 0 writer must exist for inspect output");
        *self.writer.lock().expect("poisoned") = Some(writer);

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
        _demultiplex_info: &OptDemultiplex,
    ) -> anyhow::Result<(FastQBlocksCombined, bool)> {
        let mut collected = self.collected.load(std::sync::atomic::Ordering::Relaxed);
        if collected >= self.n {
            return Ok((block, true));
        }

        let mut collector = self.collector.lock().expect("collector mutex poisoned");
        let mut iter = block.get_pseudo_iter_including_tag();
        let name_read = |read: &WrappedFastQRead, read_idx: usize| {
            let mut out = read.name().to_vec();
            for (key, values) in &block.tags {
                let str_key: &str = key.as_ref();
                out.push(b' ');
                out.extend_from_slice(str_key.as_bytes());
                out.push(b'=');
                out.extend_from_slice(&values.to_bstr(read_idx));
            }
            out
        };
        let mut read_idx = 0;
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
                            name_read(segment_read, read_idx),
                            segment_read.seq().to_vec(),
                            segment_read.qual().to_vec(),
                            tag,
                        ));
                    }
                }
                SegmentIndexOrAll::Indexed(idx) => {
                    let segment_read = &read.segments[idx.as_index()];
                    collector[0].push((
                        name_read(segment_read, read_idx),
                        segment_read.seq().to_vec(),
                        segment_read.qual().to_vec(),
                        tag,
                    ));
                }
            }

            collected += 1; //count per molecule, not per segment
            read_idx += 1;
        }
        self.collected
            .store(collected, std::sync::atomic::Ordering::Relaxed);
        Ok((block, true))
    }
    fn finalize(&self, _demultiplex_info: &OptDemultiplex) -> Result<Option<FinalizeReportResult>> {
        let collector = self.collector.lock().expect("collector mutex poisoned");
        let collected = self.collected.load(std::sync::atomic::Ordering::Relaxed);
        let mut writer = self
            .writer
            .lock()
            .expect("writer mutex poisoned")
            .take()
            .expect("writer must be set during initialization");

        if !collector.is_empty() {
            let reads_to_write = collected.min(self.n);
            let mut buf = Vec::with_capacity(256);
            match self.format {
                FileFormat::None | FileFormat::Fastq | FileFormat::Text => {
                    for read_idx in 0..reads_to_write {
                        for segment_reads in collector.iter() {
                            if let Some((name, seq, qual, tag)) = segment_reads.get(read_idx) {
                                buf.clear();
                                buf.push(b'@');
                                buf.extend_from_slice(name);
                                if let Some(demux_names) = &self.demultiplex_names
                                    && let Some(demux_name) = demux_names.get(tag)
                                {
                                    buf.extend_from_slice(b" _Demultiplex=");
                                    buf.extend_from_slice(demux_name.as_bytes());
                                }
                                buf.push(b'\n');
                                buf.extend_from_slice(seq);
                                buf.extend_from_slice(b"\n+\n");
                                buf.extend_from_slice(qual);
                                buf.push(b'\n');
                                writer.write_text_record(&buf)?;
                            } // cov:excl-line
                        }
                    }
                }
                FileFormat::Fasta => {
                    for read_idx in 0..reads_to_write {
                        for segment_reads in collector.iter() {
                            if let Some((name, seq, _qual, tag)) = segment_reads.get(read_idx) {
                                buf.clear();
                                buf.push(b'>');
                                buf.extend_from_slice(name);
                                if let Some(demux_names) = &self.demultiplex_names
                                    && let Some(demux_name) = demux_names.get(tag)
                                {
                                    buf.extend_from_slice(b" Demultiplex=");
                                    buf.extend_from_slice(demux_name.as_bytes());
                                }
                                buf.push(b'\n');
                                buf.extend_from_slice(seq);
                                buf.push(b'\n');
                                writer.write_text_record(&buf)?;
                            } // cov:excl-line
                        }
                    }
                }
                // cov:excl-start
                FileFormat::Bam => {
                    panic!("Bam not valid - should have been caught in verify");
                } // cov:excl-stop
            }
        } // cov:excl-line

        let _ = writer.finish()?;
        Ok(None)
    }
}
