use anyhow::{Context, Result, bail};
use std::path::PathBuf;
use std::{fs, io::Read, path::Path};

use super::parsers;
use super::reads::SegmentsCombined;
use crate::config::STDIN_MAGIC_PATH;

pub enum InputFile {
    Fastq(ex::fs::File),
    Fasta(ex::fs::File),
    Bam(ex::fs::File, PathBuf),
    RapidgzipFastq {
        filename: std::path::PathBuf,
        thread_count: usize,
        index_gzip: bool,
    }, //we need the filename to get the index file.
}

impl InputFile {
    pub fn get_parser(
        self,
        target_reads_per_block: usize,
        buffer_size: usize,
        options: &crate::config::InputOptions,
    ) -> Result<Box<dyn parsers::Parser>> {
        match self {
            InputFile::Fastq(file) => Ok(Box::new(parsers::FastqParser::new(
                file,
                target_reads_per_block,
                buffer_size,
            )?)),
            InputFile::Fasta(file) => {
                let fake_quality = options
                    .fasta_fake_quality
                    .context("input.options.fasta_fake_quality must be set for FASTA inputs")?;
                let parser = parsers::FastaParser::new(file, target_reads_per_block, fake_quality)?;
                Ok(Box::new(parser))
            }
            InputFile::Bam(file, _) => {
                let include_mapped = options
                    .bam_include_mapped
                    .context("input.options.bam_include_mapped must be set for BAM inputs")?;
                let include_unmapped = options
                    .bam_include_unmapped
                    .context("input.options.bam_include_unmapped must be set for BAM inputs")?;
                let parser = parsers::BamParser::new(
                    file,
                    target_reads_per_block,
                    include_mapped,
                    include_unmapped,
                )?;
                Ok(Box::new(parser))
            }
            InputFile::RapidgzipFastq {
                filename,
                thread_count,
                index_gzip,
            } => {
                let file = spawn_rapidgzip(&filename, thread_count, index_gzip)?;
                Ok(Box::new(parsers::FastqParser::new(
                    vec![file],
                    target_reads_per_block,
                    buffer_size,
                )))
            }
        }
    }
}

pub struct InputFiles {
    pub segment_files: SegmentsCombined<Vec<InputFile>>,
    pub total_size_of_largest_segment: Option<usize>,
    pub largest_segment_idx: usize,
}

pub fn total_file_size(readers: &Vec<InputFile>) -> Option<usize> {
    let mut total = 0;
    for reader in readers {
        let file = match &reader {
            InputFile::Fastq(f) => f,
            InputFile::Fasta(f) => f,
            InputFile::Bam(f, _) => f,
        };
        match file.metadata() {
            Ok(metadata) => {
                total += metadata.len() as usize;
            }
            Err(_) => {
                return None;
            }
        }
    }
    Some(total)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedInputFormat {
    Fastq,
    Fasta,
    Bam,
}

pub fn detect_input_format(path: &Path) -> Result<DetectedInputFormat> {
    if path == Path::new(STDIN_MAGIC_PATH) {
        return Ok(DetectedInputFormat::Fastq);
    }
    if let Ok(metadata) = fs::metadata(path) {
        //this is a band aid.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if metadata.file_type().is_fifo() {
                return Ok(DetectedInputFormat::Fastq);
            }
        }
    }

    let file = open_file(path)?;
    let (mut reader, _format) = niffler::send::get_reader(Box::new(file))?;
    let mut buf = [0u8; 4];
    let bytes_read = reader.read(&mut buf)?;
    if bytes_read >= 4 && &buf[..4] == b"BAM\x01" {
        return Ok(DetectedInputFormat::Bam);
    }
    if bytes_read >= 1 {
        match buf[0] {
            b'>' => return Ok(DetectedInputFormat::Fasta),
            b'@' => return Ok(DetectedInputFormat::Fastq),
            _ => {}
        }
    }
    bail!("Could not detect input format for {path:?}. Expected FASTA, FASTQ, or BAM.",);
}

pub fn open_file(filename: impl AsRef<Path>) -> Result<ex::fs::File> {
    let fh = ex::fs::File::open(filename.as_ref())
        .context(format!("Could not open file {:?}", filename.as_ref()))?;
    Ok(fh)
}

pub fn open_input_file(
    filename: impl AsRef<Path>,
    compression_format: Option<crate::config::CompressionFormat>,
    thread_count: usize,
    index_gzip: bool,
) -> Result<InputFile> {
    let filename = filename.as_ref();
    if filename.to_string_lossy() == STDIN_MAGIC_PATH {
        let file = open_stdin()?;
        return Ok(InputFile::Fastq(file));
    }
    let path = Path::new(filename);
    let format = detect_input_format(path)?;

    // Check if we should use rapidgzip
    if compression_format == Some(crate::config::CompressionFormat::Rapidgzip) {
        if format != DetectedInputFormat::Fastq {
            bail!(
                "Rapidgzip format is only supported for FastQ files. File {} appears to be {:?}",
                path.display(),
                format
            );
        }
        return Ok(InputFile::RapidgzipFastq {
            filename: path.to_path_buf(),
            thread_count,
            index_gzip,
        });
    }

    let file = open_file(path)?;
    let input_file = match format {
        DetectedInputFormat::Fastq => InputFile::Fastq(file),
        DetectedInputFormat::Fasta => InputFile::Fasta(file),
        DetectedInputFormat::Bam => InputFile::Bam(file, path.to_owned()),
    };
    Ok(input_file)
}

pub fn sum_file_sizes(filenames: &[impl AsRef<Path>]) -> Result<u64> {
    let mut total_size = 0u64;
    for filename in filenames {
        let metadata = fs::metadata(filename.as_ref()).with_context(|| {
            format!(
                "Could not get file metadata for size calculation of {:?}",
                filename.as_ref()
            )
        })?;
        total_size = total_size
            .checked_add(metadata.len())
            .with_context(|| "Total size of input files exceeds u64 max")?;
    }
    Ok(total_size)
}

pub fn open_input_files(
    input_config: &crate::config::Input,
    thread_count: usize,
) -> Result<InputFiles> {
    let compression_format = input_config.options.format;
    let index_gzip = input_config.options.index_gzip.unwrap_or(false);

    // Calculate thread count per segment
    let num_segments = input_config.segment_count();
    let threads_per_segment = if thread_count <= 2 || num_segments == 0 {
        1
    } else {
        std::cmp::max(1, (thread_count - 2) / num_segments)
    };

    match input_config.structured.as_ref().unwrap() {
        crate::config::StructuredInput::Interleaved {
            files,
            segment_order,
        } => {
            let readers: Result<Vec<_>> = files
                .iter()
                .map(|x| {
                    open_input_file(x, compression_format, threads_per_segment, index_gzip)
                        .with_context(|| {
                            format!("Problem in interleaved segment while opening '{x}'")
                        })
                })
                .collect();
            let readers = vec![readers?];
            let total_size_of_largest_segment =
                total_file_size(&readers[0]).map(|x| (x / segment_order.len()));

            Ok(InputFiles {
                segment_files: SegmentsCombined { segments: readers },
                total_size_of_largest_segment: total_size_of_largest_segment,
                largest_segment_idx: 0, // does not matter.
            })
        }
        crate::config::StructuredInput::Segmented {
            segment_order,
            segment_files,
        } => {
            let mut segments = Vec::new();
            let mut sizes = Vec::new();
            for key in segment_order {
                let filenames = segment_files
                    .get(key)
                    .expect("Segment order / segments mismatch");
                let readers: Result<Vec<_>> = filenames
                    .iter()
                    .map(|x| {
                        open_input_file(x, compression_format, threads_per_segment, index_gzip)
                            .with_context(|| {
                                format!("Problem in segment {key} while opening '{x}'")
                            })
                    })
                    .collect();
                let readers = readers?;
                sizes.push(total_file_size(&readers));
                segments.push(readers);
            }
            let total_size_of_largest_segment = sizes.iter().filter_map(|x| *x).max();
            let largest_segment_idx = sizes
                .iter()
                .filter_map(|x| *x)
                .enumerate()
                .max_by_key(|&(_idx, size)| size)
                .map(|(idx, _size)| idx)
                .unwrap_or(0);
            Ok(InputFiles {
                segment_files: SegmentsCombined { segments },
                total_size_of_largest_segment,
                largest_segment_idx,
            })
        }
    }
}

fn open_stdin() -> Result<ex::fs::File> {
    #[cfg(unix)]
    {
        use anyhow::Context as _;
        ex::fs::File::open("/dev/stdin").context("Failed to access stdin via /dev/stdin")
    }
    #[cfg(windows)]
    {
        bail!("Stdin input is not supported on windows. PRs welcome");
    }
    #[cfg(not(any(unix, windows)))]
    {
        bail!(
            "(input): '{STDIN_MAGIC_PATH}' is not supported on this platform (unknown stdio semantics)."
        );
    }
}

/// Spawns a rapidgzip process to decompress a gzipped file
fn spawn_rapidgzip(
    filename: &Path,
    thread_count: usize,
    index_gzip: bool,
) -> Result<ex::fs::File> {
    use std::process::{Command, Stdio};

    // Check for index file
    let index_path = format!("{}.rapidgzip_index", filename.display());
    let has_index = std::path::Path::new(&index_path).exists();

    // Build rapidgzip command
    let mut cmd = Command::new("rapidgzip");
    cmd.arg("--decompress")
        .arg("--stdout")
        .arg("--threads")
        .arg(thread_count.to_string())
        .arg(filename)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    // Use index if it exists
    if has_index {
        cmd.arg("--import-index").arg(&index_path);
    }

    // Export index if requested and it doesn't exist
    if index_gzip && !has_index {
        cmd.arg("--export-index").arg(&index_path);
    }

    let mut child = cmd.spawn().context(format!(
        "Failed to spawn rapidgzip process for file: {}",
        filename.display()
    ))?;

    let stdout = child
        .stdout
        .take()
        .context("Failed to capture rapidgzip stdout")?;

    // Convert the stdout pipe to an ex::fs::File
    // We need to use the file descriptor directly
    #[cfg(unix)]
    {
        use std::os::unix::io::{FromRawFd, IntoRawFd};
        let raw_fd = stdout.into_raw_fd();
        // SAFETY: We own the file descriptor from the child process stdout
        let file = unsafe { ex::fs::File::from_raw_fd(raw_fd) };
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        bail!("Rapidgzip is only supported on Unix systems");
    }
}
