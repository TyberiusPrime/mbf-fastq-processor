use anyhow::{bail, Context, Result};
use std::{fs, io::Read, path::Path};

use super::parsers;
use super::reads::SegmentsCombined;

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
                let parser =
                    parsers::FastaParser::new(vec![file], target_reads_per_block, fake_quality)?;
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
    bail!(
        "Could not detect input format for {path:?}. Expected FASTA, FASTQ, or BAM.",
    );
}

pub fn open_file(filename: impl AsRef<Path>) -> Result<ex::fs::File> {
    let fh = ex::fs::File::open(filename.as_ref())
        .context(format!("Could not open file {:?}", filename.as_ref()))?;
    Ok(fh)
}

fn create_input_file(filename: &str) -> Result<InputFile> {
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
                    create_input_file(x).with_context(|| {
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
                        create_input_file(x).with_context(|| {
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
