use anyhow::{Context, Result, bail};
use std::{fs, io::Read, path::Path};

use super::parsers;
use super::reads::SegmentsCombined;
use crate::config::STDIN_MAGIC_PATH;

pub enum InputFile {
    Fastq(ex::fs::File),
    Fasta(ex::fs::File),
    Bam(ex::fs::File),
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
                let parser = parsers::FastaParser::new(
                    vec![file],
                    target_reads_per_block,
                    buffer_size,
                    fake_quality,
                )?;
                Ok(Box::new(parser))
            }
            InputFile::Bam(file) => {
                let include_mapped = options
                    .bam_include_mapped
                    .context("input.options.bam_include_mapped must be set for BAM inputs")?;
                let include_unmapped = options
                    .bam_include_unmapped
                    .context("input.options.bam_include_unmapped must be set for BAM inputs")?;
                let parser = parsers::BamParser::new(
                    vec![file],
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
        DetectedInputFormat::Bam => InputFile::Bam(file),
    };
    Ok(input_file)
}

pub fn open_input_files(input_config: &crate::config::Input) -> Result<InputFiles> {
    match input_config.structured.as_ref().unwrap() {
        crate::config::StructuredInput::Interleaved { files, .. } => {
            let readers: Result<Vec<_>> = files
                .iter()
                .map(|x| {
                    open_input_file(x).with_context(|| {
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
                        open_input_file(x).with_context(|| {
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
        return ex::fs::File::open("/dev/stdin").context("Failed to access stdin via /dev/stdin");
    }
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        use std::io;
        use std::os::windows::io::FromRawHandle;

        const STD_INPUT_HANDLE: u32 = 0xFFFF_FFF6;
        const INVALID_HANDLE_VALUE: *mut c_void = -1isize as *mut c_void;

        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if handle.is_null() || handle == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error())
                    .context("Failed to acquire Windows stdin handle");
            }
            // Safety: We obtain ownership of the raw handle returned by GetStdHandle.
            let file = ex::fs::File::from_raw_handle(handle);
            Ok(file)
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        bail!(
            "(input): '{STDIN_MAGIC_PATH}' is not supported on this platform (unknown stdio semantics)."
        );
    }
}

#[cfg(windows)]
extern "system" {
    fn GetStdHandle(n_std_handle: u32) -> *mut std::ffi::c_void;
}
