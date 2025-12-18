use anyhow::{Context, Result, anyhow};
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
}

impl OutputWriter<'_> {
    fn finish(mut self) -> (Option<String>, Option<String>) {
        self.flush().expect("Flushing file failed");
        match self {
            OutputWriter::File(inner) => inner.finish(),
        }
    }
}

impl std::io::Write for OutputWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            OutputWriter::File(inner) => inner.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            OutputWriter::File(inner) => inner.flush(),
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

pub struct OutputFileConfig {
    directory: PathBuf,
    basename: String,
    suffix: String,
    format: FileFormat,
    compression: CompressionFormat,
    do_uncompressed_hash: bool,
    do_compressed_hash: bool,
    compression_level: Option<u8>,
    compression_threads: Option<usize>,
    simulated_failure: Option<SimulatedWriteFailure>,
    chunk_size: Option<usize>,
    chunk_index: usize,
    chunk_digit_count: usize,
    fragments_written_in_chunk: usize,
}

pub struct OutputFile<'a> {
    config: OutputFileConfig,
    handle: OutputFileHandle<'a>,
}

impl OutputFile<'_> {
    fn rotate_chunk(&mut self) -> Result<()> {
        //capture the old name
        let old_filename = self.config.filename();
        let old_handle =
            std::mem::replace(&mut self.handle, OutputFileHandle::TemporarilyOutOfAction);
        //create the hash files
        old_handle.finish(&old_filename)?;

        //now rotate the filenames, rename files if necessary,
        let new_filename = self.config.rotate_chunk()?;
        let handle = ex::fs::File::create(&new_filename).with_context(|| {
            format!(
                "Could not open file for hashing: {}",
                new_filename.display()
            )
        })?;
        //swap teh new handle in
        self.handle = self.config.build_writer(handle)?;
        //let old_handle = std::mem::replace(&mut self.handle, self.config.build_writer(handle)?);
        //and make sure teh hashes get where they need to go
        Ok(())
    }

    fn after_text_fragment(&mut self, buffer: &mut Vec<u8>) -> Result<()> {
        if let Some(chunk_size) = self.config.chunk_size {
            self.config.fragments_written_in_chunk += 1;
            if self.config.fragments_written_in_chunk >= chunk_size {
                if !buffer.is_empty() {
                    match &mut self.handle {
                        OutputFileHandle::Fastq(writer) | OutputFileHandle::Fasta(writer) => {
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
        if let Some(chunk_size) = self.config.chunk_size {
            self.config.fragments_written_in_chunk += 1;
            if self.config.fragments_written_in_chunk >= chunk_size {
                self.rotate_chunk()?;
            }
        }
        Ok(())
    }
}

enum OutputFileHandle<'a> {
    Fastq(OutputWriter<'a>), //todo: unify to text.
    Fasta(OutputWriter<'a>),
    Bam(crate::io::BamOutput<'a>),
    TemporarilyOutOfAction,
}

impl OutputFileHandle<'_> {
    fn finish(self, filename: &Path) -> Result<()> {
        match self {
            Self::Fastq(writer) | Self::Fasta(writer) => {
                let (uncompressed_hash, compressed_hash) = writer.finish();

                if let Some(hash) = uncompressed_hash {
                    Self::write_hash_file_static(filename, &hash, ".uncompressed.sha256")?;
                }
                if let Some(hash) = compressed_hash {
                    Self::write_hash_file_static(filename, &hash, ".compressed.sha256")?;
                }
                Ok(())
            }
            Self::Bam(mut bam_output) => {
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
            Self::TemporarilyOutOfAction => {
                unreachable!()
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

impl OutputFileConfig {
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
        compression_threads: Option<usize>,
        simulated_failure: Option<&SimulatedWriteFailure>,
        allow_overwrite: bool,
        chunk_size: Option<usize>,
    ) -> Result<Self> {
        let directory = directory.as_ref().to_owned();
        let filename = Self::make_filename(
            basename,
            chunk_size,
            0,
            usize::from(chunk_size.is_some()),
            suffix,
            &directory,
        );
        Self::ensure_writable(&filename, allow_overwrite, chunk_size)?;
        Ok(Self {
            directory,
            basename: basename.to_string(),
            suffix: suffix.to_string(),
            format,
            compression,
            do_uncompressed_hash,
            do_compressed_hash,
            compression_level,
            compression_threads,
            simulated_failure: simulated_failure.cloned(),
            chunk_size,
            chunk_index: 0,
            chunk_digit_count: usize::from(chunk_size.is_some()),
            fragments_written_in_chunk: 0,
        })
    }

    fn new_stdout(
        format: FileFormat,
        compression: CompressionFormat,
        do_uncompressed_hash: bool,
        do_compressed_hash: bool,
        compression_level: Option<u8>,
        compression_threads: Option<usize>,
    ) -> Result<Self> {
        match format {
            FileFormat::Fastq | FileFormat::Fasta => {}
            FileFormat::Bam => anyhow::bail!("BAM output is not supported on stdout"),
            FileFormat::None => unreachable!("Cannot emit 'none' format to stdout"),
        };
        Ok(Self {
            directory: PathBuf::new(),
            basename: "stdout".to_string(),
            suffix: String::new(),
            format,
            compression,
            do_uncompressed_hash,
            do_compressed_hash,
            compression_level,
            compression_threads,
            simulated_failure: None,
            chunk_size: None,
            chunk_index: 0,
            chunk_digit_count: 0,
            fragments_written_in_chunk: 0,
        })
    }

    fn to_writer<'a>(self) -> Result<OutputFile<'a>> {
        let handle = ex::fs::File::create(&self.filename()).with_context(|| {
            format!(
                "Could not open file for output: {}",
                self.filename().display()
            )
        })?;
        let handle = self.build_writer(handle)?;
        Ok(OutputFile {
            config: self,
            handle,
        })
    }

    fn make_filename(
        basename: &str,
        chunk_size: Option<usize>,
        chunk_index: usize,
        chunk_digit_count: usize,
        suffix: &str,
        directory: &Path,
    ) -> PathBuf {
        let mut name = basename.to_string();
        if chunk_size.is_some() {
            if !name.is_empty() {
                name.push('.');
            }
            let digits = format!("{:0width$}", chunk_index, width = chunk_digit_count.max(1));
            name.push_str(&digits);
        }
        if !suffix.is_empty() {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(&suffix);
        }
        directory.join(name)
    }

    fn ensure_writable(
        filename: &PathBuf,
        allow_overwrite: bool,
        chunk_size: Option<usize>,
    ) -> Result<()> {
        let metadata = ensure_output_destination_available(&filename, allow_overwrite)?;
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

        if is_fifo && chunk_size.is_some() {
            anyhow::bail!(
                "Chunked output is not supported when writing to named pipes: {}",
                filename.display()
            );
        }
        Ok(())
    }

    fn build_writer<'a>(&self, file_handle: ex::fs::File) -> Result<OutputFileHandle<'a>> {
        let kind = match self.format {
            FileFormat::Bam => OutputFileHandle::Bam(build_bam_output(
                file_handle,
                self.do_compressed_hash,
                self.compression_level,
                self.simulated_failure.as_ref(),
            )?),
            FileFormat::Fastq => {
                OutputFileHandle::Fastq(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    self.compression,
                    self.do_uncompressed_hash,
                    self.do_compressed_hash,
                    self.compression_level,
                    self.compression_threads,
                    self.simulated_failure.clone(),
                )?))
            }
            FileFormat::Fasta => {
                OutputFileHandle::Fasta(OutputWriter::File(HashedAndCompressedWriter::new(
                    file_handle,
                    self.compression,
                    self.do_uncompressed_hash,
                    self.do_compressed_hash,
                    self.compression_level,
                    self.compression_threads,
                    self.simulated_failure.clone(),
                )?))
            }
            FileFormat::None => unreachable!("Cannot create output file with format 'None'"),
        };
        Ok(kind)
    }

    fn filename(&self) -> PathBuf {
        Self::make_filename(
            &self.basename,
            self.chunk_size,
            self.chunk_index,
            self.chunk_digit_count,
            &self.suffix,
            &self.directory,
        )
    }

    fn rotate_chunk(&mut self) -> Result<PathBuf> {
        assert!(
            self.chunk_size.is_some(),
            "Rotate_chunk called on unrotatable output"
        );
        self.fragments_written_in_chunk = 0;
        self.chunk_index += 1;
        if self.chunk_size.is_some()
            && self.chunk_index
                >= 10usize.pow(
                    u32::try_from(self.chunk_digit_count)
                        .expect("chunk_digit_count should fit in u32"),
                )
        {
            self.chunk_digit_count += 1;
            self.rename_existing_files()?;
        }
        let new_filename = Self::make_filename(
            &self.basename,
            self.chunk_size,
            self.chunk_index,
            self.chunk_digit_count,
            &self.suffix,
            &self.directory,
        );
        Ok(new_filename)
    }

    fn rename_existing_files(&self) -> Result<()> {
        let old_chunk_digit_count = self.chunk_digit_count - 1;
        let min_value = 0;
        let max_value = 10usize.pow(
            u32::try_from(old_chunk_digit_count).expect("old_chunk_digit_count should fit in u32"),
        );
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
                if let Some(fname) = path.file_name().and_then(|s| s.to_str())
                    && fname.starts_with(
                        old_filename_prefix
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .as_ref(),
                    )
                {
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
pub struct OutputFastqs<T> {
    interleaved_file: Option<T>,
    // in input.segments_order!
    segment_files: Vec<Option<T>>,
}

impl Default for OutputFastqs<OutputFileConfig> {
    fn default() -> Self {
        OutputFastqs {
            interleaved_file: None,
            segment_files: Vec::new(),
        }
    }
}

impl OutputFastqs<OutputFileConfig> {
    pub fn to_writer<'a>(self) -> Result<OutputFastqs<OutputFile<'a>>> {
        Ok(OutputFastqs {
            interleaved_file: match self.interleaved_file {
                Some(config) => Some(config.to_writer()?),
                None => None,
            },
            segment_files: self
                .segment_files
                .into_iter()
                .map(|opt_config| match opt_config {
                    Some(config) => Ok(Some(config.to_writer()?)),
                    None => Ok(None),
                })
                .collect::<Result<Vec<Option<OutputFile<'a>>>>>()?,
        })
    }
}

impl OutputFastqs<OutputFile<'_>> {
    pub fn finish(&mut self) -> Result<()> {
        if let Some(interleaved) = self.interleaved_file.take() {
            interleaved
                .handle
                .finish(interleaved.config.filename().as_ref())?;
        }
        for file in self.segment_files.iter_mut().filter_map(Option::take) {
            file.handle.finish(file.config.filename().as_ref())?;
        }
        Ok(())
    }
}

pub struct OutputReports {
    pub html: Option<BufWriter<ex::fs::File>>,
    pub json: Option<BufWriter<ex::fs::File>>,
}

#[allow(clippy::fn_params_excessive_bools)]
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
        })
    }
}

#[allow(clippy::too_many_lines)]
fn open_one_set_of_output_files(
    parsed_config: &Config,
    output_directory: &Path,
    infix: Option<&str>,
    allow_overwrite: bool,
) -> Result<OutputFastqs<OutputFileConfig>> {
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
                        Some(OutputFileConfig::new_stdout(
                            output_config.format,
                            output_config.compression,
                            false,
                            false,
                            output_config.compression_level,
                            output_config.compression_threads,
                        )?)
                    } else if let Some(interleaved_segments) = &output_config.interleave {
                        //interleaving is handled by outputing both to the read1 output
                        ////interleaving requires read2 to be set, checked in validation
                        let interleaved_basename = join_nonempty(
                            vec![prefix.as_str(), infix_str, "interleaved"],
                            &ix_separator,
                        );
                        let interleave_count = interleaved_segments.len();
                        Some(OutputFileConfig::new_file(
                            output_directory,
                            &interleaved_basename,
                            &suffix,
                            output_config.format,
                            output_config.compression,
                            include_uncompressed_hashes,
                            include_compressed_hashes,
                            output_config.compression_level,
                            output_config.compression_threads,
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
                                Some(OutputFileConfig::new_file(
                                    output_directory,
                                    &basename,
                                    &suffix,
                                    output_config.format,
                                    output_config.compression,
                                    include_uncompressed_hashes,
                                    include_compressed_hashes,
                                    output_config.compression_level,
                                    output_config.compression_threads,
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

pub struct OutputFiles {
    pub output_segments:
        BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs<OutputFileConfig>>>>,
    pub output_reports: OutputReports,
}

pub struct OutputFilesReadyToWrite<'a> {
    pub output_segments:
        BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs<OutputFile<'a>>>>>,
    pub output_reports: OutputReports,
}

impl OutputFiles {
    pub fn to_writer<'a>(self) -> Result<OutputFilesReadyToWrite<'a>> {
        let mut output_segments = BTreeMap::new();
        for (k, v) in self.output_segments {
            let inner = Arc::try_unwrap(v)
                .map_err(|_| anyhow!("Arc had multiple references"))?
                .into_inner()
                .map_err(|_| anyhow!("Mutex was poisoned"))?;
            output_segments.insert(k, Arc::new(Mutex::new(inner.to_writer()?)));
        }
        Ok(OutputFilesReadyToWrite {
            output_segments,
            output_reports: self.output_reports,
        })
    }
}

#[allow(clippy::fn_params_excessive_bools)]
pub fn open_output_files(
    parsed_config: &Config,
    output_directory: &Path,
    demultiplexed: &OptDemultiplex,
    report_html: bool,
    report_json: bool,
    allow_overwrite: bool,
) -> Result<OutputFiles> {
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
            let mut res: BTreeMap<
                crate::demultiplex::Tag,
                Arc<Mutex<OutputFastqs<OutputFileConfig>>>,
            > = BTreeMap::new();
            let mut seen: BTreeMap<String, Arc<Mutex<OutputFastqs<OutputFileConfig>>>> =
                BTreeMap::new();
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
    output_files: &mut BTreeMap<crate::demultiplex::Tag, Arc<Mutex<OutputFastqs<OutputFile<'_>>>>>,
    interleave_order: &[usize],
    demultiplexed: &OptDemultiplex,
    buffer_size: usize,
) -> Result<()> {
    block.sanity_check()?; // runs independend if we actually output or not!
    match demultiplexed {
        OptDemultiplex::No => {
            output_block_demultiplex(
                block,
                output_files
                    .get_mut(&0)
                    .expect("default output file (tag 0) must exist"),
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
    output_files: &mut Arc<Mutex<OutputFastqs<OutputFile<'_>>>>,
    interleave_order: &[usize],
    tag: Option<crate::demultiplex::Tag>,
    buffer_size: usize,
) -> Result<()> {
    let mut buffer = Vec::with_capacity(buffer_size);
    let mut of = output_files
        .lock()
        .expect("mutex lock should not be poisoned");
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
    output_file: &mut OutputFile,
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
            match &mut output_file.handle {
                OutputFileHandle::Fastq(writer) | OutputFileHandle::Fasta(writer) => {
                    writer.write_all(buffer)?;
                }
                _ => unreachable!("Text block writer expected"),
            }
            buffer.clear();
        }
        output_file.after_text_fragment(buffer)?;
    }

    match &mut output_file.handle {
        OutputFileHandle::Fastq(writer) | OutputFileHandle::Fasta(writer) => {
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
    output_file: &mut OutputFile,
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
                    match &mut output_file.handle {
                        OutputFileHandle::Fastq(writer) | OutputFileHandle::Fasta(writer) => {
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
    match &mut output_file.handle {
        OutputFileHandle::Fastq(writer) | OutputFileHandle::Fasta(writer) => {
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
    output_file: &mut OutputFile,
    block: Option<&io::FastQBlock>,
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
) -> Result<()> {
    match output_file.config.format {
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
    output_file: &mut OutputFile,
    blocks_to_interleave: &[&io::FastQBlock],
    buffer: &mut Vec<u8>,
    buffer_size: usize,
    demultiplex_tag: Option<crate::demultiplex::Tag>,
    output_tags: Option<&Vec<crate::demultiplex::Tag>>,
) -> Result<()> {
    match output_file.config.format {
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
    output_file: &mut OutputFile,
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
        let OutputFileHandle::Bam(bam_output) = &mut output_file.handle else {
            unreachable!("BAM writer expected");
        };
        io::write_read_to_bam(bam_output, &read, 0, 1)?;
        output_file.after_bam_fragment()?;
    }

    Ok(())
}

fn write_interleaved_blocks_to_bam(
    output_file: &mut OutputFile,
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
                    let OutputFileHandle::Bam(bam_output) = &mut output_file.handle else {
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
    let template = include_str!("./html/template.html");
    let chartjs = include_str!("./html/chart/chart.umd.min.js");
    let html = template
        .replace("%TITLE%", "mbf-fastq-processor-report")
        .replace("\"%DATA%\"", json_report_string)
        .replace("/*%CHART%*/", chartjs);

    output_file.write_all(html.as_bytes())?;
    Ok(())
}
