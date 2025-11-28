use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use super::join_nonempty;
use crate::config::{CompressionFormat, Config, FileFormat};
use crate::demultiplex::OptDemultiplex;
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

pub fn ensure_output_destination_available(
    path: &Path,
    allow_overwrite: bool,
) -> Result<Option<std::fs::Metadata>> {
    use std::io::ErrorKind;

    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::FileTypeExt;

                if metadata.file_type().is_fifo() {
                    return Ok(Some(metadata));
                }
            }

            if allow_overwrite {
                return Ok(Some(metadata));
            }

            anyhow::bail!(
                "Output file '{}' already exists, refusing to overwrite. Pass --allow-overwrite to ignore this error.",
                path.display(),
            );
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
        Err(err) => {
            Err(err).with_context(|| format!("Could not inspect existing path: {}", path.display()))
        }
    }
}

pub struct OutputFile<'a> {
    directory: PathBuf,
    basename: String,
    suffix: String,
    filename: PathBuf,
    kind: OutputFileKind<'a>,
    format: FileFormat,
    compression: CompressionFormat,
    do_uncompressed_hash: bool,
    do_compressed_hash: bool,
    compression_level: Option<u8>,
    simulated_failure: Option<SimulatedWriteFailure>,
    allow_overwrite: bool,
    chunk_size: Option<usize>,
    chunk_index: usize,
    chunk_digit_count: usize,
    fragments_written_in_chunk: usize,
}

enum OutputFileKind<'a> {
    Fastq(OutputWriter<'a>),
    Fasta(OutputWriter<'a>),
    Bam(crate::io::BamOutput<'a>),
    Closed,
}

impl<'a> OutputFile<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new_file(
        directory: impl AsRef<Path>,
        basename: &str,
        suffix: &str,
        format: FileFormat,
        compression: CompressionFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
        simulated_failure: Option<&SimulatedWriteFailure>,
        allow_overwrite: bool,
        chunk_size: Option<usize>,
    ) -> Result<Self> {
        let directory = directory.as_ref().to_owned();
        let mut file = OutputFile {
            directory,
            basename: basename.to_string(),
            suffix: suffix.to_string(),
            filename: PathBuf::new(),
            kind: OutputFileKind::Closed,
            format,
            compression,
            do_uncompressed_hash,
            do_compressed_hash,
            compression_level,
            simulated_failure: simulated_failure.cloned(),
            allow_overwrite,
            chunk_size,
            chunk_index: 0,
            chunk_digit_count: usize::from(chunk_size.is_some()),
            fragments_written_in_chunk: 0,
        };
        let (filename, kind) = file.build_writer()?;
        file.filename = filename;
        file.kind = kind;
        Ok(file)
    }

    fn make_filename(&self) -> PathBuf {
        let mut name = self.basename.clone();
        if self.chunk_size.is_some() {
            if !name.is_empty() {
                name.push('.');
            }
            let digits = format!(
                "{:0width$}",
                self.chunk_index,
                width = self.chunk_digit_count.max(1)
            );
            name.push_str(&digits);
        }
        if !self.suffix.is_empty() {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&self.suffix);
        }
        self.directory.join(name)
    }

    fn build_writer(&self) -> Result<(PathBuf, OutputFileKind<'a>)> {
        let filename = self.make_filename();
        let metadata = ensure_output_destination_available(&filename, self.allow_overwrite)?;
        #[cfg(not(unix))]
        let _ = &metadata;
        #[cfg(unix)]
        let is_fifo = {
            use std::os::unix::fs::FileTypeExt;

            metadata
                .as_ref()
                .is_some_and(|meta| meta.file_type().is_fifo())
        };
        #[cfg(not(unix))]
        let is_fifo = false;

        if is_fifo && self.chunk_size.is_some() {
            anyhow::bail!(
                "Chunked output is not supported when writing to named pipes: {}",
                filename.display()
            );
        }

        let file_handle = if is_fifo {
            ex::fs::OpenOptions::new()
                .write(true)
                .open(&filename)
                .with_context(|| {
                    format!(
                        "Could not open named pipe for output: {}",
                        filename.display()
                    )
                })?
        } else {
            ex::fs::File::create(&filename)
                .with_context(|| format!("Could not open output file: {}", filename.display()))?
        };
        let kind = match self.format {
            FileFormat::Bam => OutputFileKind::Bam(build_bam_output(
                file_handle,
                self.do_compressed_hash,
                self.compression_level,
                self.simulated_failure.as_ref(),
            )?),
            FileFormat::Fastq => {
                OutputFileKind::Fastq(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    self.compression,
                    self.do_uncompressed_hash,
                    self.do_compressed_hash,
                    self.compression_level,
                    self.simulated_failure.clone(),
                )?))
            }
            FileFormat::Fasta => {
                OutputFileKind::Fasta(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    self.compression,
                    self.do_uncompressed_hash,
                    self.do_compressed_hash,
                    self.compression_level,
                    self.simulated_failure.clone(),
                )?))
            }
            FileFormat::None => unreachable!("Cannot create output file with format 'None'"),
        };
        Ok((filename, kind))
    }

    fn rotate_chunk(&mut self) -> Result<()> {
        if self.chunk_size.is_none() {
            return Ok(());
        }
        let old_filename = self.filename.clone();
        let old_kind = std::mem::replace(&mut self.kind, OutputFileKind::Closed);
        Self::finish_kind(&old_filename, old_kind)?;
        self.fragments_written_in_chunk = 0;
        self.chunk_index += 1;
        if self.chunk_size.is_some()
            && self.chunk_index >= 10usize.pow(u32::try_from(self.chunk_digit_count).unwrap())
        {
            self.chunk_digit_count += 1;
            self.rename_existing_files()?;
        }
        let (filename, kind) = self.build_writer()?;
        self.filename = filename;
        self.kind = kind;
        Ok(())
    }

    fn rename_existing_files(&self) -> Result<()> {
        let old_chunk_digit_count = self.chunk_digit_count - 1;
        let min_value = 0;
        let max_value = 10usize.pow(u32::try_from(old_chunk_digit_count).unwrap());
        let mut old_files = Vec::new();
        for entry in ex::fs::read_dir(&self.directory).with_context(|| {
            format!(
                "Could not read output directory for renaming files: {}",
                self.directory.display()
            )
        })? {
            let entry = entry.with_context(|| {
                format!(
                    "Could not read output directory entry for renaming files: {}",
                    self.directory.display()
                )
            })?;
            let path = entry.path();
            old_files.push(path);
        }
        for ii in min_value..max_value {
            let old_filename_prefix = self
                .directory
                .join(format!("{}.{ii:0old_chunk_digit_count$}", self.basename));
            let new_filename_prefix = self.directory.join(format!(
                "{}.{ii:0width$}",
                self.basename,
                width = self.chunk_digit_count
            ));
            //now find all files starting with old_prefix, rename them into new_prefix
            for path in &old_files {
                if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                    if fname.starts_with(
                        old_filename_prefix
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .as_ref(),
                    ) {
                        let suffix = &fname[old_filename_prefix
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .len()..];
                        let new_filename = new_filename_prefix.with_file_name(format!(
                            "{}{}",
                            new_filename_prefix
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy(),
                            suffix
                        ));
                        ex::fs::rename(path, &new_filename).with_context(|| {
                            format!(
                                "Could not rename output chunk file from {} to {}",
                                path.display(),
                                new_filename.display()
                            )
                        })?;
                    }
                }
            }
        }
        Ok(())
    }

    fn after_text_fragment(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        if let Some(chunk_size) = self.chunk_size {
            self.fragments_written_in_chunk += 1;
            if self.fragments_written_in_chunk >= chunk_size {
                if !buffer.is_empty() {
                    match &mut self.kind {
                        OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
                            writer.write_all(buffer)?;
                        }
                        _ => unreachable!("Text fragment encountered non-text writer"),
                    }
                    buffer.clear();
                }
                self.rotate_chunk()?;
            }
        }
        Ok(())
    }

    fn after_bam_fragment(&mut self) -> Result<()> {
        if let Some(chunk_size) = self.chunk_size {
            self.fragments_written_in_chunk += 1;
            if self.fragments_written_in_chunk >= chunk_size {
                self.rotate_chunk()?;
            }
        }
        Ok(())
    }

    fn finish_kind(filename: &Path, kind: OutputFileKind<'_>) -> Result<()> {
        match kind {
            OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
                let (uncompressed_hash, compressed_hash) = writer.finish();

                if let Some(hash) = uncompressed_hash {
                    Self::write_hash_file_static(filename, &hash, ".uncompressed.sha256")?;
                }
                if let Some(hash) = compressed_hash {
                    Self::write_hash_file_static(filename, &hash, ".compressed.sha256")?;
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
                    Self::write_hash_file_static(filename, &hash, ".uncompressed.sha256")?;
                }
                if let Some(hash) = compressed_hash {
                    Self::write_hash_file_static(filename, &hash, ".compressed.sha256")?;
                }
                Ok(())
            }
            OutputFileKind::Closed => Ok(()),
        }
    }
    fn new_stdout(
        format: FileFormat,
        compression: CompressionFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
    ) -> Result<Self> {
        let filename = PathBuf::from("stdout");
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
        Ok(OutputFile {
            directory: PathBuf::new(),
            basename: "stdout".to_string(),
            suffix: String::new(),
            filename,
            kind,
            format,
            compression,
            do_uncompressed_hash,
            do_compressed_hash,
            compression_level,
            simulated_failure: None,
            allow_overwrite: true,
            chunk_size: None,
            chunk_index: 0,
            chunk_digit_count: 0,
            fragments_written_in_chunk: 0,
        })
    }

    fn finish(self) -> Result<()> {
        let filename = self.filename.clone();
        let kind = self.kind;
        Self::finish_kind(&filename, kind)
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
    pub timing: Option<BufWriter<ex::fs::File>>,
}

impl OutputReports {
    fn new(
        output_directory: &Path,
        prefix: &String,
        report_html: bool,
        report_json: bool,
        report_timing: bool,
        allow_overwrite: bool,
    ) -> Result<OutputReports> {
        let timing = if report_timing {
            let timing_filename = output_directory.join(format!("{prefix}.timing.json"));
            let handle = ex::fs::File::create(&timing_filename).with_context(|| {
                format!(
                    "Could not open timing output file: {}",
                    timing_filename.display()
                )
            })?;
            Some(BufWriter::new(handle))
        } else {
            None
        };

        Ok(OutputReports {
            html: if report_html {
                let filename = output_directory.join(format!("{prefix}.html"));
                let _ = ensure_output_destination_available(&filename, allow_overwrite)?;
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
                let _ = ensure_output_destination_available(&filename, allow_overwrite)?;
                Some(BufWriter::new(
                    ex::fs::File::create(&filename).with_context(|| {
                        format!("Could not open output file: {}", filename.display())
                    })?,
                ))
            } else {
                None
            },
            timing,
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
    let ix_separator = parsed_config.get_ix_separator();
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
                    } else if let Some(interleaved_segments) = &output_config.interleave {
                        //interleaving is handled by outputing both to the read1 output
                        ////interleaving requires read2 to be set, checked in validation
                        let interleaved_basename = join_nonempty(
                            vec![prefix.as_str(), infix_str, "interleaved"],
                            &ix_separator,
                        );
                        let interleave_count = interleaved_segments.len();
                        Some(OutputFile::new_file(
                            output_directory,
                            &interleaved_basename,
                            &suffix,
                            output_config.format,
                            output_config.compression,
                            include_uncompressed_hashes,
                            include_compressed_hashes,
                            output_config.compression_level,
                            simulated_failure.as_ref(),
                            allow_overwrite,
                            // when interleaving chunk size is molecule count for the interleaved
                            // files
                            // so the you end up with with the same number of files if you mix
                            // interleaved and non-interleaved output
                            output_config.chunksize.map(|x| x * interleave_count),
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
                                    output_directory,
                                    &basename,
                                    &suffix,
                                    output_config.format,
                                    output_config.compression,
                                    include_uncompressed_hashes,
                                    include_compressed_hashes,
                                    output_config.compression_level,
                                    simulated_failure.as_ref(),
                                    allow_overwrite,
                                    output_config.chunksize,
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
    pub output_segments: BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs<'a>>>>,
    pub output_reports: OutputReports,
}

pub fn open_output_files<'a>(
    parsed_config: &Config,
    output_directory: &Path,
    demultiplexed: &OptDemultiplex,
    report_html: bool,
    report_json: bool,
    report_timing: bool,
    allow_overwrite: bool,
) -> Result<OutputFiles<'a>> {
    let output_reports = match &parsed_config.output {
        Some(output_config) => OutputReports::new(
            output_directory,
            &output_config.prefix,
            report_html,
            report_json,
            report_timing,
            allow_overwrite,
        )?,
        None => OutputReports {
            html: None,
            json: None,
            timing: None,
        },
    };
    match demultiplexed {
        OptDemultiplex::No => {
            let output_files = open_one_set_of_output_files(
                parsed_config,
                output_directory,
                None,
                allow_overwrite,
            )?;
            Ok(OutputFiles {
                output_segments: vec![(0, Arc::new(Mutex::new(output_files)))]
                    .into_iter()
                    .collect(),
                output_reports,
            })
        }
        OptDemultiplex::Yes(demultiplex_info) => {
            let mut res: BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs>>> =
                BTreeMap::new();
            let mut seen: BTreeMap<String, Arc<Mutex<OutputFastqs>>> = BTreeMap::new();
            for (tag, output_key) in &demultiplex_info.tag_to_name {
                if let Some(output_key) = output_key {
                    if seen.contains_key(output_key) {
                        res.insert(*tag, seen[output_key].clone());
                    } else {
                        let output = Arc::new(Mutex::new(open_one_set_of_output_files(
                            parsed_config,
                            output_directory,
                            Some(output_key),
                            allow_overwrite,
                        )?));
                        seen.insert(output_key.to_string(), output.clone());
                        res.insert(*tag, output);
                    }
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
    //that's one set of OutputFastqs per (demultiplexd) output
    output_files: &mut BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs>>>,
    interleave_order: &[usize],
    demultiplexed: &OptDemultiplex,
    buffer_size: usize,
) -> Result<()> {
    block.sanity_check()?; // runs independend if we actually output or not!
    match demultiplexed {
        OptDemultiplex::No => {
            output_block_demultiplex(
                block,
                output_files.get_mut(&0).unwrap(),
                interleave_order,
                None,
                buffer_size,
            )?;
        }
        OptDemultiplex::Yes(_demultiplex_info) => {
            for (tag, output_files) in output_files.iter_mut() {
                output_block_demultiplex(
                    block,
                    output_files,
                    interleave_order,
                    Some(*tag),
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
    tag: Option<crate::demultiplex::Tag>,
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
    output_file: &mut OutputFile<'_>,
    block: &io::FastQBlock,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
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
            match &mut output_file.kind {
                OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
                    writer.write_all(buffer)?;
                }
                _ => unreachable!("Text block writer expected"),
            }
            buffer.clear();
        }
        output_file.after_text_fragment(buffer)?;
    }

    match &mut output_file.kind {
        OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
            if !buffer.is_empty() {
                writer.write_all(buffer)?;
                buffer.clear();
            }
        }
        _ => unreachable!("Text block writer expected"),
    }
    Ok(())
}

fn write_interleaved_text_block<F>(
    output_file: &mut OutputFile<'_>,
    blocks_to_interleave: &[&io::FastQBlock],
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
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
                    match &mut output_file.kind {
                        OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
                            writer.write_all(buffer)?;
                        }
                        _ => unreachable!("Text block writer expected"),
                    }
                    buffer.clear();
                }
                output_file.after_text_fragment(buffer)?;
            } else {
                break 'outer;
            }
        }
    }
    match &mut output_file.kind {
        OutputFileKind::Fastq(writer) | OutputFileKind::Fasta(writer) => {
            if !buffer.is_empty() {
                writer.write_all(buffer)?;
                buffer.clear();
            }
        }
        _ => unreachable!("Text block writer expected"),
    }
    Ok(())
}

#[allow(clippy::redundant_closure_for_method_calls)] // can't go WrappedFastQRead::as_fasta - lifetime issues
fn output_block_inner(
    output_file: &mut OutputFile<'_>,
    block: Option<&io::FastQBlock>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
) -> Result<()> {
    match output_file.format {
        FileFormat::Fastq => write_text_block(
            output_file,
            block.expect("FASTQ output requires a block"),
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.append_as_fastq(out),
        ),
        FileFormat::Fasta => write_text_block(
            output_file,
            block.expect("FASTA output requires a block"),
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.as_fasta(out),
        ),
        FileFormat::Bam => {
            let block = block.expect("BAM output requires a block");
            write_block_to_bam(output_file, block, demultiplex_tag, output_tags)?;
            buffer.clear();
            Ok(())
        }
        FileFormat::None => unreachable!("Cannot output with format 'None'"),
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::redundant_closure_for_method_calls)] // can't go WrappedFastQRead::as_fasta - lifetime issues
fn output_block_interleaved(
    output_file: &mut OutputFile<'_>,
    blocks_to_interleave: &[&io::FastQBlock],
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
) -> Result<()> {
    match output_file.format {
        FileFormat::Fastq => write_interleaved_text_block(
            output_file,
            blocks_to_interleave,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.append_as_fastq(out),
        ),
        FileFormat::Fasta => write_interleaved_text_block(
            output_file,
            blocks_to_interleave,
            buffer,
            buffer_size,
            demultiplex_tag,
            output_tags,
            |read, out| read.as_fasta(out),
        ),
        FileFormat::Bam => {
            write_interleaved_blocks_to_bam(
                output_file,
                blocks_to_interleave,
                demultiplex_tag,
                output_tags,
            )?;
            buffer.clear();
            Ok(())
        }
        FileFormat::None => unreachable!("Cannot output with format 'None'"),
    }
}

fn write_block_to_bam(
    output_file: &mut OutputFile<'_>,
    block: &io::FastQBlock,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
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
        let OutputFileKind::Bam(bam_output) = &mut output_file.kind else {
            unreachable!("BAM writer expected");
        };
        io::write_read_to_bam(bam_output, &read, 0, 1)?;
        output_file.after_bam_fragment()?;
    }

    Ok(())
}

fn write_interleaved_blocks_to_bam(
    output_file: &mut OutputFile<'_>,
    blocks_to_interleave: &[&io::FastQBlock],
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
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
                    let OutputFileKind::Bam(bam_output) = &mut output_file.kind else {
                        unreachable!("BAM writer expected")
                    };
                    io::write_read_to_bam(bam_output, &read, segment_index, segment_count)?;
                    output_file.after_bam_fragment()?;
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
            "repository": env!("CARGO_PKG_HOMEPAGE"),
        }),
    );
    let reports = report_collector.lock().unwrap();
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
            key.as_str().unwrap().to_string(),
            report_output.remove(key.as_str().unwrap()).unwrap(),
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
    let template = include_str!("./html/template.html");
    let chartjs = include_str!("./html/chart/chart.umd.min.js");
    let html = template
        .replace("%TITLE%", "mbf-fastq-processor-report")
        .replace("\"%DATA%\"", json_report_string)
        .replace("/*%CHART%*/", chartjs);

    output_file.write_all(html.as_bytes())?;
    Ok(())
}
