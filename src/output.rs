use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use super::join_nonempty;
use crate::config::{CompressionFormat, Config, FileFormat};
use crate::demultiplex::Demultiplexed;
use crate::io::{
    self,
    compressed_output::{HashedAndCompressedWriter, SimulatedWriteFailure},
};
use crate::transformations::FinalizeReportResult;
use noodles::{bam, bgzf, sam};

pub struct OutputRunMarker {
    pub path: PathBuf,
    preexisting: bool,
}

impl OutputRunMarker {
    pub fn create(output_directory: &Path, prefix: &str) -> Result<Self> {
        let path = output_directory.join(format!("{prefix}.incompleted"));
        let preexisting = std::fs::symlink_metadata(&path).is_ok();
        let mut file = ex::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)
            .with_context(|| {
                format!("Could not open completion marker file: {}", path.display())
            })?;
        file.write_all(b"run incomplete\n")?;
        file.sync_all()
            .with_context(|| format!("Failed to sync completion marker: {}", path.display()))?;
        Ok(OutputRunMarker { path, preexisting })
    }

    pub fn mark_complete(&self) -> Result<()> {
        match ex::fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err).with_context(|| {
                format!(
                    "Failed to remove completion marker after completion: {}",
                    self.path.display()
                )
            }),
        }
    }

    pub fn preexisting(&self) -> bool {
        self.preexisting
    }
}

enum OutputWriter<'a> {
    File(HashedAndCompressedWriter<'a, ex::fs::File>),
    Stdout(HashedAndCompressedWriter<'a, std::io::Stdout>),
}

impl OutputWriter<'_> {
    fn finish(mut self) -> (Option<String>, Option<String>) {
        self.flush().expect("Flushing file failed");
        match self {
            OutputWriter::File(inner) => inner.finish(),
            OutputWriter::Stdout(inner) => inner.finish(),
        }
    }
}

impl std::io::Write for OutputWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputWriter::File(inner) => inner.write(buf),
            OutputWriter::Stdout(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputWriter::File(inner) => inner.flush(),
            OutputWriter::Stdout(inner) => inner.flush(),
        }
    }
}

pub fn ensure_output_destination_available(path: &Path, allow_overwrite: bool) -> Result<()> {
    use std::io::ErrorKind;

    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;

                if metadata.file_type().is_fifo() {
                    return Ok(());
                }
            }

            if allow_overwrite {
                return Ok(());
            }

            anyhow::bail!(
                "Output file '{}' already exists, refusing to overwrite",
                path.display()
            );
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
        Err(err) => {
            Err(err).with_context(|| format!("Could not inspect existing path: {}", path.display()))
        }
    }
}

pub struct OutputFile<'a> {
    filename: PathBuf,
    kind: OutputFileKind<'a>,
}

enum OutputFileKind<'a> {
    Fastq(OutputWriter<'a>),
    Fasta(OutputWriter<'a>),
    Bam(crate::io::BamOutput<'a>),
}

impl OutputFile<'_> {
    fn new_file(
        filename: impl AsRef<Path>,
        format: FileFormat,
        compression: CompressionFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
        simulated_failure: Option<&SimulatedWriteFailure>,
        allow_overwrite: bool,
    ) -> Result<Self> {
        let filename = filename.as_ref().to_owned();
        ensure_output_destination_available(&filename, allow_overwrite)?;
        let file_handle = ex::fs::File::create(&filename)
            .with_context(|| format!("Could not open output file: {}", filename.display()))?;
        let kind = match format {
            FileFormat::Bam => OutputFileKind::Bam(build_bam_output(
                file_handle,
                do_compressed_hash,
                compression_level,
                simulated_failure,
            )?),
            FileFormat::Fastq => {
                OutputFileKind::Fastq(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    compression,
                    do_uncompressed_hash,
                    do_compressed_hash,
                    compression_level,
                    simulated_failure.cloned(),
                )?))
            }
            FileFormat::Fasta => {
                OutputFileKind::Fasta(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    compression,
                    do_uncompressed_hash,
                    do_compressed_hash,
                    compression_level,
                    simulated_failure.cloned(),
                )?))
            }
            FileFormat::None => unreachable!("Cannot create output file with format 'None'"),
        };
        Ok(OutputFile {
            filename: filename.clone(),
            kind,
        })
    }
    fn new_stdout(
        format: FileFormat,
        compression: CompressionFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
    ) -> Result<Self> {
        let filename = "stdout".into();
        let file_handle = std::io::stdout();
        let writer = HashedAndCompressedWriter::new(
            file_handle,
            compression,
            do_uncompressed_hash,
            do_compressed_hash,
            compression_level,
            None,
        )?;
        let kind = match format {
            FileFormat::Fastq => OutputFileKind::Fastq(OutputWriter::Stdout(writer)),
            FileFormat::Fasta => OutputFileKind::Fasta(OutputWriter::Stdout(writer)),
            FileFormat::Bam => anyhow::bail!("BAM output is not supported on stdout"),
            FileFormat::None => unreachable!("Cannot emit 'none' format to stdout"),
        };
        Ok(OutputFile { filename, kind })
    }

    fn finish(self) -> Result<()> {
        // First flush the writer to complete any compression
        match self.kind {
            OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
                let (uncompressed_hash, compressed_hash) = writer.finish();

                if let Some(hash) = uncompressed_hash {
                    Self::write_hash_file_static(&self.filename, &hash, ".uncompressed.sha256")?;
                }
                if let Some(hash) = compressed_hash {
                    Self::write_hash_file_static(&self.filename, &hash, ".compressed.sha256")?;
                }
                Ok(())
            }
            OutputFileKind::Bam(mut bam_output) => {
                bam_output
                    .writer
                    .try_finish()
                    .context("Failed to finish BAM writer")?;
                let bgzf_writer = bam_output.writer.into_inner();
                let hashed_writer = bgzf_writer.into_inner();
                let (uncompressed_hash, compressed_hash) = hashed_writer.finish();
                if let Some(hash) = uncompressed_hash {
                    Self::write_hash_file_static(&self.filename, &hash, ".uncompressed.sha256")?;
                }
                if let Some(hash) = compressed_hash {
                    Self::write_hash_file_static(&self.filename, &hash, ".compressed.sha256")?;
                }
                Ok(())
            }
        }
    }

    fn write_hash_file_static(filename: &Path, hash: &str, suffix: &str) -> Result<()> {
        let hash_filename = filename.with_file_name(format!(
            "{}{}",
            filename.file_name().unwrap_or_default().to_string_lossy(),
            suffix
        ));

        let mut fh = ex::fs::File::create(hash_filename)
            .with_context(|| format!("Could not open file for hashing: {}", filename.display()))?;
        fh.write_all(hash.as_bytes())?;
        fh.flush()?;
        Ok(())
    }
}

fn build_bam_output<'a>(
    file_handle: ex::fs::File,
    do_compressed_hash: bool,
    compression_level: Option<u8>,
    simulated_failure: Option<&SimulatedWriteFailure>,
) -> Result<io::BamOutput<'a>> {
    let hashed_writer = HashedAndCompressedWriter::new(
        file_handle,
        CompressionFormat::Uncompressed,
        false,
        do_compressed_hash,
        None,
        simulated_failure.cloned(),
    )?;

    let bgzf_writer = match compression_level {
        Some(level) => {
            let level = bgzf::io::writer::CompressionLevel::try_from(level)
                .context("Invalid compression level for BAM BGZF writer")?;
            bgzf::io::writer::Builder::default()
                .set_compression_level(level)
                .build_from_writer(hashed_writer)
        }
        None => bgzf::io::Writer::new(hashed_writer),
    };

    let mut writer = bam::io::Writer::from(bgzf_writer);
    let header = Arc::new(create_unaligned_bam_header());
    writer
        .write_header(&header)
        .context("Failed to write BAM header")?;

    Ok(io::BamOutput { writer, header })
}

fn create_unaligned_bam_header() -> sam::Header {
    sam::Header::from_str(
        "@HD\tVN:1.6\tSO:unsorted\n@PG\tID:mbf-fastq-processor\tPN:mbf-fastq-processor\n",
    )
    .expect("static BAM header must parse")
}

#[derive(Default)]
pub struct OutputFastqs<'a> {
    interleaved_file: Option<OutputFile<'a>>,
    // in input.segments_order!
    segment_files: Vec<Option<OutputFile<'a>>>,
}

impl OutputFastqs<'_> {
    pub fn finish(&mut self) -> Result<()> {
        if let Some(interleaved) = self.interleaved_file.take() {
            interleaved.finish()?;
        }
        for file in self.segment_files.iter_mut().filter_map(Option::take) {
            file.finish()?;
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
                ensure_output_destination_available(&filename, allow_overwrite)?;
                Some(BufWriter::new(
                    ex::fs::File::create(&filename).with_context(|| {
                        format!("Could not open output file: {}", filename.display())
                    })?,
                ))
            } else {
                None
            },
            json: if report_json {
                let filename = output_directory.join(format!("{prefix}.json"));
                ensure_output_destination_available(&filename, allow_overwrite)?;
                Some(BufWriter::new(
                    ex::fs::File::create(&filename).with_context(|| {
                        format!("Could not open output file: {}", filename.display())
                    })?,
                ))
            } else {
                None
            },
        })
    }
}

#[allow(clippy::too_many_lines)]
fn open_one_set_of_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
    infix: Option<&str>,
    allow_overwrite: bool,
) -> Result<OutputFastqs<'a>> {
    let simulated_failure = parsed_config
        .options
        .debug_failures
        .simulated_output_failure()?;
    let ix_separator = parsed_config
        .output
        .as_ref()
        .map_or_else(crate::config::default_ix_separator, |o| {
            o.ix_separator.clone()
        });
    Ok(match &parsed_config.output {
        Some(output_config) => {
            let prefix = &output_config.prefix;
            let suffix = output_config.get_suffix();
            let include_uncompressed_hashes = output_config.output_hash_uncompressed;
            let include_compressed_hashes = output_config.output_hash_compressed;
            let (interleaved_file, segment_files) = match output_config.format {
                FileFormat::None => (None, Vec::new()),
                _ => {
                    let infix_str = infix.unwrap_or("");
                    let interleaved_file = if output_config.stdout {
                        assert!(
                            output_config.interleave.is_some(),
                            "check did not make certain interleave is set when stdout is set"
                        );
                        Some(OutputFile::new_stdout(
                            output_config.format,
                            output_config.compression,
                            false,
                            false,
                            output_config.compression_level,
                        )?)
                    } else if output_config.interleave.is_some() {
                        //interleaving is handled by outputing both to the read1 output
                        ////interleaving requires read2 to be set, checked in validation
                        let interleaved_basename = join_nonempty(
                            vec![prefix.as_str(), infix_str, "interleaved"],
                            &ix_separator,
                        );
                        Some(OutputFile::new_file(
                            output_directory.join(format!("{interleaved_basename}.{suffix}",)),
                            output_config.format,
                            output_config.compression,
                            include_uncompressed_hashes,
                            include_compressed_hashes,
                            output_config.compression_level,
                            simulated_failure.as_ref(),
                            allow_overwrite,
                        )?)
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
                                Some(OutputFile::new_file(
                                    output_directory.join(format!("{basename}.{suffix}")),
                                    output_config.format,
                                    output_config.compression,
                                    include_uncompressed_hashes,
                                    include_compressed_hashes,
                                    output_config.compression_level,
                                    simulated_failure.as_ref(),
                                    allow_overwrite,
                                )?)
                            } else {
                                None
                            });
                        }
                    }
                    (interleaved_file, segment_files)
                }
            };

            OutputFastqs {
                interleaved_file,
                segment_files,
            }
        }
        None => OutputFastqs::default(),
    })
}

pub struct OutputFiles<'a> {
    pub output_segments: Vec<Arc<Mutex<OutputFastqs<'a>>>>,
    pub output_reports: OutputReports,
}

pub fn open_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
    demultiplexed: &Demultiplexed,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,
) -> Result<OutputFiles<'a>> {
    let output_reports = match &parsed_config.output {
        Some(output_config) => OutputReports::new(
            output_directory,
            &output_config.prefix,
            report_html,
            report_json,
            allow_overwrite,
        )?,
        None => OutputReports {
            html: None,
            json: None,
        },
    };

    match demultiplexed {
        Demultiplexed::No => {
            let output_files = open_one_set_of_output_files(
                parsed_config,
                output_directory,
                None,
                allow_overwrite,
            )?;
            Ok(OutputFiles {
                output_segments: vec![Arc::new(Mutex::new(output_files))],
                output_reports,
            })
        }
        Demultiplexed::Yes(demultiplex_info) => {
            let mut res = Vec::new();
            let mut seen: HashMap<String, Arc<Mutex<OutputFastqs>>> = HashMap::new();
            for (_tag, output_key) in demultiplex_info.iter_outputs() {
                if seen.contains_key(output_key) {
                    res.push(seen[output_key].clone());
                } else {
                    let output = Arc::new(Mutex::new(open_one_set_of_output_files(
                        parsed_config,
                        output_directory,
                        Some(output_key),
                        allow_overwrite,
                    )?));
                    seen.insert(output_key.to_string(), output.clone());
                    res.push(output);
                }
            }
            Ok(OutputFiles {
                output_segments: res,
                output_reports,
            })
        }
    }
}

#[allow(clippy::if_not_else)]
pub fn output_block(
    block: &io::FastQBlocksCombined,
    output_files: &mut [Arc<Mutex<OutputFastqs>>],
    interleave_order: &[usize],
    demultiplexed: &Demultiplexed,
    buffer_size: usize,
) -> Result<()> {
    block.sanity_check()?; // runs independend if we actually output or not!
    match demultiplexed {
        Demultiplexed::No => {
            output_block_demultiplex(
                block,
                &mut output_files[0],
                interleave_order,
                None,
                buffer_size,
            )?;
        }
        Demultiplexed::Yes(demultiplex_info) => {
            for (file_no, (tag, _output_key)) in demultiplex_info.iter_outputs().enumerate() {
                let output_files = &mut output_files[file_no];
                output_block_demultiplex(
                    block,
                    output_files,
                    interleave_order,
                    Some(tag),
                    buffer_size,
                )?;
            }
        }
    }
    Ok(())
}

#[allow(clippy::if_not_else)]
fn output_block_demultiplex(
    block: &io::FastQBlocksCombined,
    output_files: &mut Arc<Mutex<OutputFastqs>>,
    interleave_order: &[usize],
    tag: Option<u16>,
    buffer_size: usize,
) -> Result<()> {
    let mut buffer = Vec::with_capacity(buffer_size);
    let mut of = output_files.lock().unwrap();
    for (segment_block, output_file) in block.segments.iter().zip(of.segment_files.iter_mut()) {
        if let Some(output_file) = output_file {
            output_block_inner(
                output_file,
                Some(segment_block),
                &mut buffer,
                buffer_size,
                tag,
                block.output_tags.as_ref(),
            )?;
        }
    }
    if let Some(interleaved_file) = &mut of.interleaved_file {
        let blocks_to_interleave: Vec<_> = interleave_order
            .iter()
            .map(|&i| &block.segments[i])
            .collect();

        output_block_interleaved(
            interleaved_file,
            &blocks_to_interleave,
            &mut buffer,
            buffer_size,
            tag,
            block.output_tags.as_ref(),
        )?;
    }
    Ok(())
}

fn write_text_block<F>(
    block: &io::FastQBlock,
    writer: &mut OutputWriter<'_>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
    mut encode: F,
) -> Result<()>
where
    for<'read> F: FnMut(&io::WrappedFastQRead<'read>, &mut Vec<u8>),
{
    let mut pseudo_iter = if let Some(demultiplex_tag) = demultiplex_tag {
        block.get_pseudo_iter_filtered_to_tag(
            demultiplex_tag,
            output_tags.expect("Demultiplex output tags missing"),
        )
    } else {
        block.get_pseudo_iter()
    };

    while let Some(read) = pseudo_iter.pseudo_next() {
        encode(&read, buffer);
        if buffer.len() > buffer_size {
            writer.write_all(buffer)?;
            buffer.clear();
        }
    }

    writer.write_all(buffer)?;
    buffer.clear();
    Ok(())
}

fn write_interleaved_text_block<F>(
    blocks_to_interleave: &[&io::FastQBlock],
    writer: &mut OutputWriter<'_>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
    mut encode: F,
) -> Result<()>
where
    for<'read> F: FnMut(&io::WrappedFastQRead<'read>, &mut Vec<u8>),
{
    let mut pseudo_iters: Vec<_> = blocks_to_interleave
        .iter()
        .map(|block| {
            if let Some(demultiplex_tag) = demultiplex_tag {
                block.get_pseudo_iter_filtered_to_tag(
                    demultiplex_tag,
                    output_tags.expect("Demultiplex output tags missing"),
                )
            } else {
                block.get_pseudo_iter()
            }
        })
        .collect();
    assert!(!pseudo_iters.is_empty(), "Interleave output but no blocks?");

    'outer: loop {
        for iter in &mut pseudo_iters {
            if let Some(entry) = iter.pseudo_next() {
                encode(&entry, buffer);

                if buffer.len() > buffer_size {
                    writer.write_all(buffer)?;
                    buffer.clear();
                }
            } else {
                break 'outer;
            }
        }
    }
    writer.write_all(buffer)?;

    buffer.clear();
    Ok(())
}

#[allow(clippy::redundant_closure_for_method_calls)] // can't go WrappedFastQRead::as_fasta - lifetime issues
fn output_block_inner(
    output_file: &mut OutputFile<'_>,
    block: Option<&io::FastQBlock>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) -> Result<()> {
    match &mut output_file.kind {
        OutputFileKind::Fastq(writer) => write_text_block(
            block.expect("FASTQ output requires a block"),
            writer,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.append_as_fastq(out),
        ),
        OutputFileKind::Fasta(writer) => write_text_block(
            block.expect("FASTA output requires a block"),
            writer,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.as_fasta(out),
        ),
        OutputFileKind::Bam(bam_output) => {
            let block = block.expect("BAM output requires a block");
            write_block_to_bam(bam_output, block, demultiplex_tag, output_tags)?;
            buffer.clear();
            Ok(())
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::redundant_closure_for_method_calls)] // can't go WrappedFastQRead::as_fasta - lifetime issues
fn output_block_interleaved(
    output_file: &mut OutputFile<'_>,
    blocks_to_interleave: &[&io::FastQBlock],
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) -> Result<()> {
    match &mut output_file.kind {
        OutputFileKind::Fastq(writer) => write_interleaved_text_block(
            blocks_to_interleave,
            writer,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.append_as_fastq(out),
        ),
        OutputFileKind::Fasta(writer) => write_interleaved_text_block(
            blocks_to_interleave,
            writer,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.as_fasta(out),
        ),
        OutputFileKind::Bam(bam_output) => {
            write_interleaved_blocks_to_bam(
                bam_output,
                blocks_to_interleave,
                demultiplex_tag,
                output_tags,
            )?;
            buffer.clear();
            Ok(())
        }
    }
}

fn write_block_to_bam(
    bam_output: &mut io::BamOutput<'_>,
    block: &io::FastQBlock,
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) -> Result<()> {
    let mut pseudo_iter = if let Some(demultiplex_tag) = demultiplex_tag {
        block.get_pseudo_iter_filtered_to_tag(
            demultiplex_tag,
            output_tags.expect("Demultiplex output tags missing"),
        )
    } else {
        block.get_pseudo_iter()
    };

    while let Some(read) = pseudo_iter.pseudo_next() {
        io::write_read_to_bam(bam_output, &read, 0, 1)?;
    }

    Ok(())
}

fn write_interleaved_blocks_to_bam(
    bam_output: &mut io::BamOutput<'_>,
    blocks_to_interleave: &[&io::FastQBlock],
    demultiplex_tag: Option<u16>,
    output_tags: Option<&Vec<u16>>,
) -> Result<()> {
    let mut pseudo_iters: Vec<_> = blocks_to_interleave
        .iter()
        .map(|block| {
            if let Some(demultiplex_tag) = demultiplex_tag {
                block.get_pseudo_iter_filtered_to_tag(
                    demultiplex_tag,
                    output_tags.expect("Demultiplex output tags missing"),
                )
            } else {
                block.get_pseudo_iter()
            }
        })
        .collect();

    let segment_count = pseudo_iters.len();
    assert!(segment_count > 0, "Interleave output but no blocks?");

    loop {
        for (segment_index, iter) in pseudo_iters.iter_mut().enumerate() {
            match iter.pseudo_next() {
                Some(read) => {
                    io::write_read_to_bam(bam_output, &read, segment_index, segment_count)?;
                }
                None => return Ok(()),
            }
        }
    }
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
            "cwd": std::env::current_dir().unwrap(),
            "input_files": input_config,
        }),
    );
    let reports = report_collector.lock().unwrap();
    for report in reports.iter() {
        let key = report_labels[report.report_no].clone();
        match output.entry(key) {
            serde_json::map::Entry::Vacant(entry) => {
                entry.insert(report.contents.clone());
            }
            serde_json::map::Entry::Occupied(mut entry) => entry.get_mut().merge(&report.contents),
        }
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
    let report_order: Vec<serde_json::Value> = report_labels
        .iter()
        .map(|label| serde_json::Value::String(label.clone()))
        .collect();
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
    let template = include_str!("../html/template.html");
    let chartjs = include_str!("../html/chart/chart.umd.min.js");
    let html = template
        .replace("%TITLE%", "mbf-fastq-processor-report")
        .replace("\"%DATA%\"", json_report_string)
        .replace("/*%CHART%*/", chartjs);

    output_file.write_all(html.as_bytes())?;
    Ok(())
}
