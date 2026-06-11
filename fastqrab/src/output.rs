use anyhow::{Context, Result, anyhow};
use fastqrab_io::blocks::FastQChunk;
use noodles::bam;
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::num::{NonZero, NonZeroUsize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use stringpod::CrossPods;

use crate::config::CheckedConfig;
use crate::demultiplex::OptDemultiplex;
use crate::transformations::FinalizeReportResult;
use fastqrab_io::FileFormat;
use fastqrab_io::ensure_output_destination_available;
use fastqrab_io::io::output::chunked_writer::{
    BamSinkOptions, ChunkPaths, ChunkPolicy, ChunkedRecordWriter, SinkConfig, WriteTarget,
};
use fastqrab_io::io::output::simulated_failure::SimulatedWriteFailure;
use fastqrab_io::io::{self, Tags};
use fastqrab_steps::join_nonempty;

/// Runtime BAM write options derived from config at file-open time.
///
/// Same shape as `BamSinkOptions`; the alias keeps the call sites readable.
pub type BamWriteOptions = BamSinkOptions;

pub struct OutputRunMarker {
    pub path: PathBuf,
    preexisting: bool,
}

impl OutputRunMarker {
    pub fn create(output_directory: &Path, prefix: &str) -> Result<Self> {
        let path = output_directory.join(format!("{prefix}.incompleted"));
        let prefix_parent = path
            .parent()
            .expect("Really expected a parent on a joined directory");
        if prefix_parent != output_directory {
            ex::fs::create_dir_all(prefix_parent).with_context(|| {
                format!(
                    "Could not create output (sub) directory for completion marker: {}",
                    prefix_parent.display()
                )
            })?;
        }
        let preexisting = std::fs::symlink_metadata(&path).is_ok();
        let mut file = ex::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    // I mean, we just created it, so I don't expect it to fail
                    // cov:excl-start
                    let parent_dir = path.parent().unwrap_or(output_directory);
                    anyhow!("Output directory does not exist: {}", parent_dir.display())
                    // cov:excl-stop
                } else {
                    e.into()
                }
            })
            .with_context(|| {
                format!("Could not open completion marker file: {}", path.display())
            })?;
        file.write_all(b"run incomplete\n")?;
        file.sync_all()
            .with_context(|| format!("Failed to sync completion marker: {}", path.display()))?;
        Ok(OutputRunMarker { path, preexisting })
    }

    #[mutants::skip] // it's only precaution
    pub fn mark_complete(&self) -> Result<()> {
        match ex::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            // cov:excl-start
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| {
                format!(
                    "Failed to remove completion marker after completion: {}",
                    self.path.display()
                )
            }),
            // cov:excl-stop
        }
    }

    pub fn was_preexisting(&self) -> bool {
        self.preexisting
    }
}

/// Captures everything required to construct a `ChunkedRecordWriter`. Built
/// during the orchestrator phase (`open_one_set_of_output_files`); turned into
/// an actual `ChunkedRecordWriter` later via [`OutputStreamConfig::build`].
pub struct OutputStreamConfig {
    target: WriteTarget,
    format: FileFormat,
    sink_config: SinkConfig,
    chunk_policy: ChunkPolicy,
    bam_options: Option<BamSinkOptions>,
    bam_thread_count: NonZero<usize>,
    allow_overwrite: bool,
}

impl OutputStreamConfig {
    #[expect(clippy::too_many_arguments, reason = "all needed up-front")]
    fn new_file(
        directory: impl AsRef<Path>,
        basename: &str,
        suffix: &str,
        format: FileFormat,
        sink_config: SinkConfig,
        chunk_policy: ChunkPolicy,
        bam_options: BamSinkOptions,
        bam_thread_count: NonZero<usize>,
        allow_overwrite: bool,
    ) -> Result<Self> {
        let paths = ChunkPaths {
            directory: directory.as_ref().to_owned(),
            basename: basename.to_owned(),
            suffix: suffix.to_owned(),
        };
        let digit = usize::from(chunk_policy.records_per_chunk.is_some());
        let first = paths.nth(0, digit);
        let metadata = ensure_output_destination_available(&first, allow_overwrite)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            let is_fifo = metadata.as_ref().is_some_and(|m| m.file_type().is_fifo());
            if is_fifo && chunk_policy.records_per_chunk.is_some() {
                anyhow::bail!(
                    "Chunked output is not supported when writing to named pipes: {}",
                    first.display()
                );
            }
        }
        #[cfg(not(unix))]
        let _ = metadata;

        Ok(Self {
            target: WriteTarget::Files(paths),
            format,
            sink_config,
            chunk_policy,
            bam_options: Some(bam_options),
            bam_thread_count,
            allow_overwrite,
        })
    }

    fn new_stdout(format: FileFormat, sink_config: SinkConfig) -> Self {
        match format {
            FileFormat::Fastq | FileFormat::Fasta => {}
            //cov:excl-start
            FileFormat::Bam => unreachable!(
                "BAM output is not supported on stdout. Should have been caught in validation. Bug."
            ),
            //cov:excl-stop
            FileFormat::Text | FileFormat::None => {
                unreachable!("Cannot emit 'text' or 'none' format to stdout via this path") //cov:excl-line
            }
        }
        Self {
            target: WriteTarget::Stdout,
            format,
            sink_config,
            chunk_policy: ChunkPolicy::default(),
            bam_options: None,
            bam_thread_count: NonZero::<usize>::new(1).expect("1 != 0"),
            allow_overwrite: false,
        }
    }

    fn build(self) -> Result<ChunkedRecordWriter> {
        ChunkedRecordWriter::new(
            self.format,
            self.target,
            self.sink_config,
            self.chunk_policy,
            self.bam_options,
            self.bam_thread_count,
            self.allow_overwrite,
        )
    }
}

#[derive(Default)]
pub struct OutputFastqs<T> {
    interleaved_file: Option<T>,
    // in input.segments_order!
    segment_files: Vec<Option<T>>,
}

impl OutputFastqs<OutputStreamConfig> {
    pub fn into_writer(self) -> Result<OutputFastqs<ChunkedRecordWriter>> {
        Ok(OutputFastqs {
            interleaved_file: self
                .interleaved_file
                .map(OutputStreamConfig::build)
                .transpose()?,
            segment_files: self
                .segment_files
                .into_iter()
                .map(|opt| opt.map(OutputStreamConfig::build).transpose())
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

impl OutputFastqs<ChunkedRecordWriter> {
    /// Total records written across all segment/interleaved files for this tag.
    pub fn total_fragment_written(&self) -> u64 {
        self.segment_files
            .iter()
            .filter_map(Option::as_ref)
            .map(ChunkedRecordWriter::total_records_written)
            .chain(
                self.interleaved_file
                    .as_ref()
                    .map(ChunkedRecordWriter::total_records_written),
            )
            .max()
            .unwrap_or(0)
    }

    pub fn finish(&mut self) -> Result<()> {
        if let Some(interleaved) = self.interleaved_file.take() {
            let _ = interleaved.finish()?;
        }
        for file in self.segment_files.iter_mut().filter_map(Option::take) {
            let _ = file.finish()?;
        }
        Ok(())
    }
}

pub struct OutputReports {
    pub html: Option<BufWriter<ex::fs::File>>,
    pub json: Option<BufWriter<ex::fs::File>>,
}

impl OutputReports {
    fn new(
        output_directory: &Path,
        prefix: &String,
        report_html: bool,
        report_json: bool,
        allow_overwrite: bool,
    ) -> Result<OutputReports> {
        Ok(OutputReports {
            html: if report_html {
                let filename = output_directory.join(format!("{prefix}.html"));
                let _ = ensure_output_destination_available(&filename, allow_overwrite)?;
                Some(BufWriter::new(
                    ex::fs::File::create(&filename).with_context(|| {
                        // cov:excl-start
                        format!("Could not open output file: {}", filename.display())
                        // cov:excl-stop
                    })?, // cov:excl-line
                ))
            } else {
                None
            },
            json: if report_json {
                let filename = output_directory.join(format!("{prefix}.json"));
                let _ = ensure_output_destination_available(&filename, allow_overwrite)?;
                Some(BufWriter::new(
                    ex::fs::File::create(&filename).with_context(|| {
                        // cov:excl-start
                        format!("Could not open output file: {}", filename.display())
                        // cov:excl-stop
                    })?, // cov:excl-line
                ))
            } else {
                None
            },
        })
    }
}

/// Resolve the `BamWriteOptions` from the config at runtime.
///
/// This is called when opening output files (not during config validation) because
/// resolving reference sequences from a barcodes section or BAM file requires the
/// fully-verified `CheckedConfig`.
fn resolve_bam_write_options(
    output_config: &fastqrab_steps::config::Output,
    barcodes: &indexmap::IndexMap<fastqrab_config::TagLabel, fastqrab_steps::config::Barcodes>,
) -> Result<BamWriteOptions> {
    let bam_opts = output_config.bam.as_ref();

    let comment_separation_char = bam_opts.map_or(b' ', |b| b.comment_separation_char);

    let tag_to_bam_tags: Vec<([u8; 2], String)> = bam_opts
        .map(|b| {
            b.tag_to_bam_tag
                .iter()
                .map(|(tag_label, bam_tag)| (bam_tag.0, tag_label.to_string()))
                .collect()
        })
        .unwrap_or_default();

    let (tag_to_reference, reference_sequences) = if let Some(tag_to_ref) =
        bam_opts.and_then(|b| b.tag_to_reference.as_ref())
    {
        let tag_name = tag_to_ref.tag.clone();
        let ref_seqs: Vec<(String, usize)> = if let Some(barcodes_key) =
            &tag_to_ref.references_from_barcodes
        {
            // Extract reference names from barcodes section.
            // Each unique barcode name is one reference; length = barcode sequence length.
            let label = fastqrab_config::TagLabel::Normal(barcodes_key.clone());
            let barcode_section = barcodes.get(&label).expect
                ("Barcode section not found for tag_to_reference. Should have been caught in validation.");
            // seq_to_name maps sequence -> name; collect unique names with sequence length
            let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
            let mut result: Vec<(String, usize)> = Vec::new();
            for (seq, name) in barcode_section.seq_to_name.iter() {
                if seen.insert(name.clone()) {
                    result.push((name.clone(), seq.len()));
                }
            }
            result
        } else if let Some(from_bam_path) = &tag_to_ref.references_from_bam {
            // Extract reference sequences from the BAM file header.
            let file = std::fs::File::open(from_bam_path)
                .with_context(|| format!("Could not open BAM reference file: {from_bam_path}"))?;
            let mut reader = bam::io::Reader::new(file);
            let header = reader
                .read_header()
                .with_context(|| format!("Could not read BAM header from: {from_bam_path}"))?;
            header
                .reference_sequences()
                .iter()
                .map(|(name, rs)| {
                    (
                        String::from_utf8_lossy(name).into_owned(),
                        usize::from(rs.length()),
                    )
                })
                .collect()
        } else {
            // cov:excl-start
            Vec::new()
            // cov:excl-stop
        };
        (Some(tag_name), ref_seqs)
    } else {
        (None, Vec::new())
    };

    let mut opts = BamWriteOptions {
        comment_separation_char,
        tag_to_bam_tags,
        tag_to_reference,
        reference_sequences: std::sync::Arc::new(reference_sequences),
        shared_header: None,
    };
    opts.build_shared_header();
    Ok(opts)
}

fn build_sink_config(
    output_config: &fastqrab_steps::config::Output,
    simulated_failure: Option<&SimulatedWriteFailure>,
) -> SinkConfig {
    SinkConfig {
        compression: output_config.compression,
        compression_level: output_config.compression_level,
        compression_threads: Some(NonZeroUsize::new(output_config.compression_threads).expect(
            "Config should have validated output.compression_threads > 0 \
                    when output.compression is set",
        )),
        hash_uncompressed: output_config.output_hash_uncompressed,
        hash_compressed: output_config.output_hash_compressed,
        simulated_failure: simulated_failure.cloned(),
    }
}

fn bam_thread_count(output_config: &fastqrab_steps::config::Output) -> NonZero<usize> {
    NonZeroUsize::new(output_config.compression_threads)
        .expect("Config should have validated output.compression_threads > 0")
}

fn open_one_set_of_output_files(
    parsed_config: &CheckedConfig,
    output_directory: &Path,
    infix: Option<&str>,
    allow_overwrite: bool,
    bam_write_options: BamWriteOptions,
) -> Result<OutputFastqs<OutputStreamConfig>> {
    let simulated_failure = parsed_config
        .options
        .debug_failures
        .simulated_output_failure()?;
    let ix_separator = parsed_config.get_ix_separator();
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let prefix = &output_config.prefix;
            let suffix = output_config.get_suffix();
            let sink_config = build_sink_config(output_config, simulated_failure.as_ref());
            let bam_threads = bam_thread_count(output_config);
            let (interleaved_file, segment_files) = if output_config.format == FileFormat::None {
                (None, Vec::new())
            } else {
                let infix_str = infix.unwrap_or("");
                let interleaved_file = if output_config.stdout {
                    assert!(
                        output_config.interleave.is_some(),
                        "config check did not make certain interleave is set when stdout is set"
                    );
                    // Stdout: hashing is meaningless, no chunking, BAM blocked at the
                    // OutputStreamConfig level.
                    let mut sc = sink_config.clone();
                    sc.hash_uncompressed = false;
                    sc.hash_compressed = false;
                    Some(OutputStreamConfig::new_stdout(output_config.format, sc))
                } else if let Some(interleaved_segments) = &output_config.interleave {
                    let interleaved_basename = join_nonempty(
                        vec![prefix.as_str(), infix_str, "interleaved"],
                        &ix_separator,
                    );
                    let interleave_count = interleaved_segments.len();
                    // When interleaving, chunk_size counts molecules — multiply by the number
                    // of segments so files line up with non-interleaved output.
                    let chunk_policy = ChunkPolicy {
                        records_per_chunk: output_config.chunksize.map(|x| x * interleave_count),
                    };
                    Some(OutputStreamConfig::new_file(
                        output_directory,
                        &interleaved_basename,
                        &suffix,
                        output_config.format,
                        sink_config.clone(),
                        chunk_policy,
                        bam_write_options.clone(),
                        bam_threads,
                        allow_overwrite,
                    )?) //cov:excl-line
                } else {
                    None
                };
                let mut segment_files = Vec::new();
                if let Some(output) = output_config.output.as_ref() {
                    for name in parsed_config.input.get_segment_order() {
                        segment_files.push(if output.iter().any(|x| x == name) {
                            let basename = join_nonempty(
                                vec![prefix.as_str(), infix_str, name.as_str()],
                                &ix_separator,
                            );
                            let chunk_policy = ChunkPolicy {
                                records_per_chunk: output_config.chunksize,
                            };
                            Some(OutputStreamConfig::new_file(
                                output_directory,
                                &basename,
                                &suffix,
                                output_config.format,
                                sink_config.clone(),
                                chunk_policy,
                                bam_write_options.clone(),
                                bam_threads,
                                allow_overwrite,
                            )?)
                        } else {
                            None
                        });
                    }
                }
                (interleaved_file, segment_files)
            };

            OutputFastqs {
                interleaved_file,
                segment_files,
            }
        }
        None =>
        // cov:excl-start
        {
            unreachable!("Should not be reached")
        } // cov:excl-stop
    })
}

pub struct OutputFiles {
    pub output_segments:
        BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs<OutputStreamConfig>>>>,
    pub output_reports: OutputReports,
}

pub struct OutputFilesReadyToWrite {
    pub output_segments: BTreeMap<crate::demultiplex::Tag, OutputFastqs<ChunkedRecordWriter>>,
    pub output_reports: OutputReports,
}

impl OutputFiles {
    pub fn into_writer(self) -> Result<OutputFilesReadyToWrite> {
        let mut output_segments = BTreeMap::new();
        for (k, v) in self.output_segments {
            let inner = Arc::try_unwrap(v)
                .map_err(|_e| anyhow!("Arc had multiple references:"))?
                .into_inner()
                .map_err(|err| anyhow!("Mutex was poisoned: {err:?}"))?;
            output_segments.insert(k, inner.into_writer()?);
        }
        Ok(OutputFilesReadyToWrite {
            output_segments,
            output_reports: self.output_reports,
        })
    }
}

pub fn open_output_files(
    parsed_config: &CheckedConfig,
    output_directory: &Path,
    demultiplexed: &OptDemultiplex,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,
) -> Result<OutputFiles> {
    let output_config = parsed_config.output.as_ref().expect(
        // cov:excl-start
        "Should not be reached, output config should have been checked in validation",
        // cov:excl-stop
    );
    let output_reports = OutputReports::new(
        output_directory,
        &output_config.prefix,
        report_html,
        report_json,
        allow_overwrite,
    )?; // cov:excl-line
    let bam_write_options = resolve_bam_write_options(output_config, &parsed_config.barcodes)?;
    match demultiplexed {
        OptDemultiplex::No => {
            let output_files = open_one_set_of_output_files(
                parsed_config,
                output_directory,
                None,
                allow_overwrite,
                bam_write_options,
            )?;
            Ok(OutputFiles {
                output_segments: vec![(0, Arc::new(Mutex::new(output_files)))]
                    .into_iter()
                    .collect(),
                output_reports,
            })
        }
        OptDemultiplex::Yes(demultiplex_info) => {
            let mut res: BTreeMap<
                crate::demultiplex::Tag,
                Arc<Mutex<OutputFastqs<OutputStreamConfig>>>,
            > = BTreeMap::new();
            for (tag, output_key) in &demultiplex_info.tag_to_name {
                if let Some(output_key) = output_key {
                    let output = Arc::new(Mutex::new(open_one_set_of_output_files(
                        parsed_config,
                        output_directory,
                        Some(output_key),
                        allow_overwrite,
                        bam_write_options.clone(),
                    )?)); // cov:excl-line
                    res.insert(*tag, output);
                }
            }
            Ok(OutputFiles {
                output_segments: res,
                output_reports,
            })
        }
    }
}

pub fn output_block(
    block: &io::FastQBlocksCombined,
    output_files: &mut BTreeMap<crate::demultiplex::Tag, OutputFastqs<ChunkedRecordWriter>>,
    interleave_order: &[usize],
    demultiplexed: &OptDemultiplex,
    _buffer_size: usize,
) -> Result<()> {
    block.sanity_check()?;
    match demultiplexed {
        OptDemultiplex::No => {
            output_block_demultiplex(
                block,
                output_files
                    .get_mut(&0)
                    .expect("default output file (tag 0) must exist"),
                interleave_order,
                None,
            )?;
        }
        OptDemultiplex::Yes(_) => {
            for (tag, output_files) in output_files.iter_mut() {
                output_block_demultiplex(block, output_files, interleave_order, Some(*tag))?;
                // cov:excl-line
            }
        }
    }
    Ok(())
}

fn output_block_demultiplex(
    block: &io::FastQBlocksCombined,
    output_files: &mut OutputFastqs<ChunkedRecordWriter>,
    interleave_order: &[usize],
    tag: Option<crate::demultiplex::Tag>,
) -> Result<()> {
    for (segment_block, output_file) in block
        .segments
        .iter()
        .zip(output_files.segment_files.iter_mut())
    {
        if let Some(output_file) = output_file {
            output_block_inner(
                output_file,
                segment_block,
                tag,
                block.output_tags.as_ref(),
                &block.tags,
            )?;
        }
    }
    if let Some(interleaved_file) = &mut output_files.interleaved_file {
        let blocks_to_interleave: Vec<_> = interleave_order
            .iter()
            .map(|&i| &block.segments[i])
            .collect();
        output_block_interleaved(
            interleaved_file,
            &blocks_to_interleave,
            tag,
            block.output_tags.as_ref(),
            &block.tags,
        )?; // cov:excl-line
    }
    Ok(())
}

fn output_block_inner(
    writer: &mut ChunkedRecordWriter,
    block: &FastQChunk,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
    tags: &Tags,
) -> Result<()> {
    let format = writer.format();
    let iter = block.iter_filtered_to_tag(demultiplex_tag, output_tags);

    let mut buf = Vec::<u8>::with_capacity(256);
    match format {
        FileFormat::Fastq => {
            for (_idx, read) in iter {
                buf.clear();
                read.append_as_fastq(&mut buf);
                writer.write_text_record(&buf)?;
            }
        }
        FileFormat::Fasta => {
            for (_idx, read) in iter {
                buf.clear();
                read.append_as_fasta(&mut buf);
                writer.write_text_record(&buf)?;
            }
        }
        FileFormat::Bam => {
            for (read_index, read) in iter {
                writer.write_bam_record(&read, read_index, 0, 1, tags)?;
            }
        }
        // cov:excl-start
        FileFormat::Text | FileFormat::None => {
            unreachable!("Cannot output reads with format 'Text' or 'None'")
        } // cov:excl-stop
    }
    Ok(())
}

fn output_block_interleaved(
    writer: &mut ChunkedRecordWriter,
    blocks_to_interleave: &[&FastQChunk],
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
    tags: &Tags,
) -> Result<()> {
    let format = writer.format();
    let mut iters: Vec<_> = blocks_to_interleave
        .iter()
        .map(|block| block.iter_filtered_to_tag(demultiplex_tag, output_tags))
        .collect();
    let segment_count = iters.len();
    assert!(segment_count > 0, "Interleave output but no blocks?");
    let mut buf = Vec::<u8>::with_capacity(256);
    'outer: loop {
        for (segment_index, iter) in iters.iter_mut().enumerate() {
            match format {
                FileFormat::Fastq => {
                    let Some((_read_idx, read)) = iter.next() else {
                        break 'outer;
                    };
                    buf.clear();
                    read.append_as_fastq(&mut buf);
                    writer.write_text_record(&buf)?;
                }
                FileFormat::Fasta => {
                    let Some((_read_index, read)) = iter.next() else {
                        break 'outer;
                    };
                    buf.clear();
                    read.append_as_fasta(&mut buf);
                    writer.write_text_record(&buf)?;
                }
                FileFormat::Bam => {
                    let Some((read_index, read)) = iter.next() else {
                        break 'outer;
                    };
                    writer.write_bam_record(
                        &read,
                        read_index,
                        segment_index,
                        segment_count,
                        tags,
                    )?; // cov:excl-line
                    let _ = segment_index;
                }
                // cov:excl-start
                FileFormat::Text | FileFormat::None => {
                    unreachable!("Cannot output reads with format 'Text' or 'None'")
                } // cov:excl-stop
            }
        }
    }
    Ok(())
}

pub fn output_json_report(
    output_file: Option<&mut BufWriter<ex::fs::File>>,
    report_collector: &Arc<Mutex<Vec<FinalizeReportResult>>>,
    report_labels: &[String],
    current_dir: &str,
    input_config: &crate::config::Input,
    raw_config: &str,
) -> Result<String> {
    use json_value_merge::Merge;
    let mut output: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    //store run info such as version in "__"
    output.insert(
        "__".to_string(),
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "cwd": std::env::current_dir().expect("Failed to retreive current working directory"),
            "input_files": input_config,
            "repository": env!("CARGO_PKG_HOMEPAGE"),
        }),
    );
    let reports = report_collector
        .lock()
        .expect("mutex lock should not be poisoned");
    let report_order: Vec<serde_json::Value> = report_labels
        .iter()
        .map(|label| serde_json::Value::String(label.clone()))
        .collect();

    let mut report_output: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();

    for report in reports.iter() {
        let key = report_labels[report.report_no].clone();
        match report_output.entry(key) {
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(report.contents.clone());
            }
            serde_json::map::Entry::Occupied(mut entry) => entry.get_mut().merge(&report.contents),
        }
    }

    for key in &report_order {
        output.insert(
            key.as_str()
                .expect("report keys must be strings")
                .to_string(),
            report_output
                .remove(key.as_str().expect("report keys must be strings"))
                .expect("key must exist in report_output map"),
        );
    }

    let mut run_info = serde_json::Map::new();

    run_info.insert(
        "program_version".to_string(),
        serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );

    run_info.insert(
        "input_toml".to_string(),
        serde_json::Value::String(raw_config.to_string()),
    );
    run_info.insert(
        "working_directory".to_string(),
        serde_json::Value::String(current_dir.to_string()),
    );

    output.insert("run_info".to_string(), serde_json::Value::Object(run_info));

    // Add report_order to maintain the order of reports as defined in TOML

    output.insert(
        "report_order".to_string(),
        serde_json::Value::Array(report_order),
    );

    let str_output = serde_json::to_string_pretty(&output)?;
    if let Some(output_file) = output_file {
        output_file.write_all(str_output.as_bytes())?;
    }
    Ok(str_output)
}

pub fn output_html_report(
    output_file: &mut BufWriter<ex::fs::File>,
    json_report_string: &str,
) -> Result<()> {
    if json_report_string
        .to_ascii_lowercase()
        .contains("</script>")
    {
        panic!("JSON output contained </script> which will break html parsing."); // cov:excl-line
    }
    let template = include_str!("./html/template.html");
    let chartjs = include_str!("./html/chart/chart.umd.min.js");
    let html = template
        .replace("%TITLE%", "fastqrab-report")
        .replace("\"%DATA%\"", json_report_string)
        .replace("/*%CHART%*/", chartjs);

    output_file.write_all(html.as_bytes())?;
    Ok(())
}
