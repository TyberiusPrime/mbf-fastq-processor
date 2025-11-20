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
                vec![file],
                target_reads_per_block,
                buffer_size,
            ))),
            InputFile::Fasta(file) => {
                let fake_quality = options
                    .fasta_fake_quality
                    .context("input.options.fasta_fake_quality must be set for FASTA inputs")?;
                let parser =
                    parsers::FastaParser::new(vec![file], target_reads_per_block, fake_quality)?;
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
                    vec![file],
                    vec![path],
                    target_reads_per_block,
                    include_mapped,
                    include_unmapped,
                )?;
                Ok(Box::new(parser))
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

pub fn open_input_file(filename: impl AsRef<Path>) -> Result<InputFile> {
    let filename = filename.as_ref();
    if filename.to_string_lossy() == STDIN_MAGIC_PATH {
        let file = open_stdin()?;
        return Ok(InputFile::Fastq(file));
    }
    let path = Path::new(filename);
    let format = detect_input_format(path)?;
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

pub fn open_input_files(input_config: &crate::config::Input) -> Result<InputFiles> {
    match input_config.structured.as_ref().unwrap() {
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
