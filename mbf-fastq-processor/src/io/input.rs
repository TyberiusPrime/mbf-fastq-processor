use anyhow::{Context, Result, bail};
use std::{fs, io::Read, path::{Path, PathBuf}};

use super::parsers;
use super::reads::SegmentsCombined;
use crate::config::STDIN_MAGIC_PATH;

pub struct FileSource {
    pub file: ex::fs::File,
    pub path: PathBuf,
    pub use_rapidgzip: bool,
    pub threads: usize,
}

pub enum InputFile {
    Fastq(FileSource),
    Fasta(FileSource),
    Bam(FileSource),
}

impl InputFile {
    pub fn get_parser(
        self,
        target_reads_per_block: usize,
        buffer_size: usize,
        options: &crate::config::InputOptions,
    ) -> Result<Box<dyn parsers::Parser>> {
        match self {
            InputFile::Fastq(source) => Ok(Box::new(parsers::FastqParser::new(
                vec![source],
                target_reads_per_block,
                buffer_size,
            ))),
            InputFile::Fasta(source) => {
                let fake_quality = options
                    .fasta_fake_quality
                    .context("input.options.fasta_fake_quality must be set for FASTA inputs")?;
                let parser =
                    parsers::FastaParser::new(vec![source], target_reads_per_block, fake_quality)?;
                Ok(Box::new(parser))
            }
            InputFile::Bam(source) => {
                let include_mapped = options
                    .bam_include_mapped
                    .context("input.options.bam_include_mapped must be set for BAM inputs")?;
                let include_unmapped = options
                    .bam_include_unmapped
                    .context("input.options.bam_include_unmapped must be set for BAM inputs")?;
                let parser = parsers::BamParser::new(
                    vec![source.file],
                    target_reads_per_block,
                    include_mapped,
                    include_unmapped,
                )?;
                Ok(Box::new(parser))
            }
        }
    }
}

pub type InputFiles = SegmentsCombined<Vec<InputFile>>;

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

pub fn open_input_file(filename: impl AsRef<Path>, use_rapidgzip: bool, threads_per_segment: usize) -> Result<InputFile> {
    let filename = filename.as_ref();
    if filename.to_string_lossy() == STDIN_MAGIC_PATH {
        let file = open_stdin()?;
        let source = FileSource {
            file,
            path: PathBuf::from(STDIN_MAGIC_PATH),
            use_rapidgzip: false, // stdin cannot use rapidgzip
            threads: threads_per_segment,
        };
        return Ok(InputFile::Fastq(source));
    }
    let path = Path::new(filename);
    let format = detect_input_format(path)?;
    let file = open_file(path)?;
    let source = FileSource {
        file,
        path: path.to_path_buf(),
        use_rapidgzip,
        threads: threads_per_segment,
    };
    let input_file = match format {
        DetectedInputFormat::Fastq => InputFile::Fastq(source),
        DetectedInputFormat::Fasta => InputFile::Fasta(source),
        DetectedInputFormat::Bam => InputFile::Bam(source),
    };
    Ok(input_file)
}

pub fn open_input_files(input_config: &crate::config::Input, thread_count: usize) -> Result<InputFiles> {
    // Calculate threads per segment based on total threads and segment count
    // Reserve 2 threads for other operations as requested
    let available_threads = thread_count.saturating_sub(2).max(1);
    let segment_count = input_config.segment_count();
    let threads_per_segment = if segment_count > 0 {
        (available_threads / segment_count).max(1)
    } else {
        1
    };

    let use_rapidgzip = input_config.options.use_internal_rapidgzip;

    match input_config.structured.as_ref().unwrap() {
        crate::config::StructuredInput::Interleaved { files, .. } => {
            let readers: Result<Vec<_>> = files
                .iter()
                .map(|x| {
                    open_input_file(x, use_rapidgzip, threads_per_segment).with_context(|| {
                        format!("Problem in interleaved segment while opening '{x}'")
                    })
                })
                .collect();
            Ok(SegmentsCombined {
                segments: vec![readers?],
            })
        }
        crate::config::StructuredInput::Segmented {
            segment_order,
            segment_files,
        } => {
            let mut segments = Vec::new();
            for key in segment_order {
                let filenames = segment_files
                    .get(key)
                    .expect("Segment order / segments mismatch");
                let readers: Result<Vec<_>> = filenames
                    .iter()
                    .map(|x| {
                        open_input_file(x, use_rapidgzip, threads_per_segment).with_context(|| {
                            format!("Problem in segment {key} while opening '{x}'")
                        })
                    })
                    .collect();
                segments.push(readers?);
            }
            Ok(SegmentsCombined { segments })
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
