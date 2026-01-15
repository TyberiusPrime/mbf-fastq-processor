use anyhow::{Context, Result, bail};
use ex::Wrapper;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{fs, io::Read, path::Path};

use super::parsers;
use super::reads::SegmentsCombined;
use crate::config::{CompressionFormat, STDIN_MAGIC_PATH};
use crate::io::parsers::ThreadCount;

pub enum InputFile {
    Fastq(ex::fs::File, Option<PathBuf>),
    Fasta(ex::fs::File, Option<PathBuf>),
    Bam(ex::fs::File, PathBuf),
}

#[derive(Copy, Clone)]
pub enum DecompressionOptions {
    Default,
    Rapidgzip {
        thread_count: ThreadCount,
        index_gzip: bool,
    },
}

impl InputFile {
    #[mutants::skip] // will just fall back to default decompression options, which obvs. works
    pub fn get_filename(&self) -> Option<&PathBuf> {
        match self {
            InputFile::Fastq(_, filename) | InputFile::Fasta(_, filename) => filename.as_ref(),
            InputFile::Bam(_, filename) => Some(filename),
        }
    }

    pub fn get_parser(
        self,
        target_reads_per_block: usize,
        buffer_size: usize,
        thread_count: ThreadCount,
        options: &crate::config::InputOptions,
    ) -> Result<Box<dyn parsers::Parser>> {
        let decompression_options = if options
            .use_rapidgzip
            .expect("Config.check should have set use_rapidgzip no matter what")
            && self.get_filename().is_some()
        {
            DecompressionOptions::Rapidgzip {
                thread_count,
                index_gzip: options.build_rapidgzip_index.unwrap_or(false),
            }
        } else {
            DecompressionOptions::Default
        };
        match self {
            InputFile::Fastq(file, filename) => Ok(Box::new(parsers::FastqParser::new(
                file.into_inner(),
                filename.as_ref(),
                target_reads_per_block,
                buffer_size,
                decompression_options,
            )?)),
            InputFile::Fasta(file, filename) => {
                let fake_quality = options
                    .fasta_fake_quality
                    .context("input.options.fasta_fake_quality must be set for FASTA inputs")?;
                let parser = parsers::FastaParser::new(
                    file,
                    filename.as_ref(),
                    target_reads_per_block,
                    fake_quality,
                    decompression_options,
                )?;
                Ok(Box::new(parser))
            }
            InputFile::Bam(file, path) => {
                let include_mapped = options
                    .bam_include_mapped
                    .context("input.options.bam_include_mapped must be set for BAM inputs")?;
                let include_unmapped = options
                    .bam_include_unmapped
                    .context("input.options.bam_include_unmapped must be set for BAM inputs")?;
                let parser = parsers::BamParser::new(
                    file,
                    path,
                    target_reads_per_block,
                    include_mapped,
                    include_unmapped,
                    thread_count.0,
                )?;
                Ok(Box::new(parser))
            }
        }
    }
}

pub struct InputFiles {
    pub segment_files: SegmentsCombined<Vec<InputFile>>,
    pub total_size_of_largest_segment: Option<u64>,
    pub largest_segment_idx: usize,
}

#[must_use]
pub fn total_file_size(readers: &Vec<InputFile>) -> Option<u64> {
    let mut total = 0;
    for reader in readers {
        let file = match &reader {
            InputFile::Fastq(f, __opt_filename) => f,
            InputFile::Fasta(f, _opt_filename) => f,
            InputFile::Bam(f, _) => f,
        };
        match file.metadata() {
            Ok(metadata) => {
                total += metadata.len();
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

pub fn detect_input_format(path: &Path) -> Result<(DetectedInputFormat, CompressionFormat)> {
    if path == Path::new(STDIN_MAGIC_PATH) {
        return Ok((DetectedInputFormat::Fastq, CompressionFormat::Uncompressed));
    }
    if let Ok(metadata) = fs::metadata(path) {
        //this is a band aid.
        #[cfg(unix)]
        {
            use std::os::unix::fs::FileTypeExt;
            if metadata.file_type().is_fifo() {
                return Ok((DetectedInputFormat::Fastq, CompressionFormat::Uncompressed));
            }
        }
    }

    let file = open_file(path)?;
    let (mut reader, format) =
        niffler::send::get_reader(Box::new(file)).context("Problem detecting file format")?;
    let mut buf = [0u8; 4];
    let bytes_read = reader.read(&mut buf)?;
    if bytes_read >= 4 && &buf[..4] == b"BAM\x01" {
        return Ok((DetectedInputFormat::Bam, CompressionFormat::Uncompressed));
    }
    let compression_format = match format {
        niffler::send::compression::Format::Gzip => CompressionFormat::Gzip,
        niffler::send::compression::Format::Zstd => CompressionFormat::Zstd,
        niffler::send::compression::Format::No => CompressionFormat::Uncompressed,
        _ => bail!("Unsupported compression format for input detection"),
    };
    if bytes_read >= 1 {
        match buf[0] {
            b'>' => return Ok((DetectedInputFormat::Fasta, compression_format)),
            b'@' => return Ok((DetectedInputFormat::Fastq, compression_format)),
            _ => {
                bail!(
                    "Could not detect input format for {path}. Expected FASTA, FASTQ, or BAM.",
                    path = path.display()
                );
            }
        }
    } else {
        // an empty file. We just treat it as no reads fastq and let the downstream handle
        // 0 reads gracefully
        return Ok((DetectedInputFormat::Fastq, compression_format));
    }
}

pub fn open_file(filename: impl AsRef<Path>) -> Result<ex::fs::File> {
    let fh = ex::fs::File::open(filename.as_ref()).context(format!(
        "Could not open file \"{}\"",
        filename.as_ref().display()
    ))?;
    Ok(fh)
}

pub fn open_input_file(filename: impl AsRef<Path>) -> Result<InputFile> {
    let filename = filename.as_ref();
    if filename.to_string_lossy() == STDIN_MAGIC_PATH {
        let file = open_stdin()?;
        return Ok(InputFile::Fastq(file, None));
    }
    let path = Path::new(filename);
    let format = detect_input_format(path)?.0;

    let file = open_file(path)?;
    let input_file = match format {
        DetectedInputFormat::Fastq => InputFile::Fastq(file, Some(path.to_owned())),
        DetectedInputFormat::Fasta => InputFile::Fasta(file, Some(path.to_owned())),
        DetectedInputFormat::Bam => InputFile::Bam(file, path.to_owned()),
    };
    Ok(input_file)
}

#[mutants::skip] // Only used for estimation of expected input size - failure means we allocate
// more ScaleableCuckooFilter but will still work
pub fn sum_file_sizes(filenames: &[impl AsRef<Path>]) -> Result<u64> {
    let mut total_size = 0u64;
    for filename in filenames {
        let metadata = fs::metadata(filename.as_ref()).with_context(|| {
            format!(
                "Could not get file metadata for size calculation of {}",
                filename.as_ref().display()
            )
        })?;
        total_size = total_size
            .checked_add(metadata.len())
            .with_context(|| "Total size of input files exceeds u64 max")?;
    }
    Ok(total_size)
}

pub fn open_input_files(input_config: &crate::config::Input) -> Result<InputFiles> {
    match input_config
        .structured
        .as_ref()
        .expect("structured input must be set after config parsing")
    {
        crate::config::StructuredInput::Interleaved {
            files,
            segment_order,
        } => {
            let readers: Result<Vec<_>> = files
                .iter()
                .map(|x| {
                    open_input_file(x).with_context(|| {
                        format!("Problem in interleaved segment while opening '{x}'")
                    })
                })
                .collect();
            let readers = vec![readers?];
            //since there is only one segment, it's by default the largest
            // mutant fp, since it only affects cuckoo buffer sizes
            let total_size_of_largest_segment =
                total_file_size(&readers[0]).map(|x| x / segment_order.len() as u64);

            Ok(InputFiles {
                segment_files: SegmentsCombined { segments: readers },
                total_size_of_largest_segment,
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
                        open_input_file(x).with_context(|| {
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
                .map_or(0, |(idx, _size)| idx);
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

#[must_use]
pub fn find_rapidgzip_in_path() -> Option<PathBuf> {
    // I know, which isn't posix
    let mut cmd = Command::new("which");
    cmd.arg("rapidgzip");
    let output = cmd.output().ok()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        dbg!("Found rapidgzip from path");
        Some(PathBuf::from(path))
    } else {
        //if not on path, but this is a nix binary, refer to the nix store one our flake added for
        dbg!("no rapidgzip in path");
        let nix_rapidgzip = option_env!("NIX_RAPIDGZIP");
        nix_rapidgzip.and_then(|p| {
            dbg!("Had nix rapid gzip");
            let path = PathBuf::from(p);
            if path.exists() {
                dbg!("Had nix rapid gzip - and it existed");
                Some(path)
            } else {
                dbg!("Had nix rapid gzip - and it did not exist");
                // probably an os without which command, such as windows. Then we likely don't have
                // rapidgzip either?
                None
            }
        })
    }
}

/// Spawns a rapidgzip process to decompress a gzipped file
pub fn spawn_rapidgzip(
    filename: &Path,
    thread_count: ThreadCount,
    index_gzip: bool,
) -> Result<std::fs::File> {
    // Check for index file
    let index_path = format!("{}.rapidgzip_index", filename.display());
    let has_index = std::path::Path::new(&index_path).exists();

    let rapidgzip_command = find_rapidgzip_in_path().unwrap_or_else(|| "rapidgzip".into());
    // Build rapidgzip command
    let mut cmd = Command::new(rapidgzip_command);
    cmd.arg("--stdout")
        .arg("-d")
        .arg("-P")
        .arg(thread_count.0.to_string())
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
        "Failed to spawn rapidgzip process for file: {}. Make sure you have a rapidgzip binary on your path.",
        filename.display()
    ))?;
    // dbg!(cmd);

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
        let file = unsafe { std::fs::File::from_raw_fd(raw_fd) };
        Ok(file)
    }
    #[cfg(not(unix))]
    {
        bail!("Rapidgzip is only supported on Unix systems");
    }
}
