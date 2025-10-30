use super::super::{FinalizeReportResult, Step, Transformation};
use crate::config::{CompressionFormat, FileFormat, SegmentIndexOrAll, SegmentOrAll};
use crate::demultiplex::Demultiplex;
use crate::io::output::compressed_output::HashedAndCompressedWriter;
use anyhow::{Result, bail};
use std::{io::Write, path::Path};

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>);

#[derive(eserde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub n: usize,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>, // needed to produce output filename

    pub infix: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: FileFormat,
    #[serde(default)]
    pub compression: CompressionFormat,
    #[serde(default)]
    pub compression_level: Option<u8>,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub collector: Vec<Vec<NameSeqQualTuple>>,
    #[serde(default)]
    #[serde(skip)]
    collected: usize,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    ix_separator: String,

    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    //we write either interleaved (one file) or one segment (one file)
    writer: Option<HashedAndCompressedWriter<'static, ex::fs::File>>,
}

impl std::fmt::Debug for Inspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Inspect")
            .field("n", &self.n)
            .field("segment", &self.segment)
            //        .field("segment_index", &self.segment_index)
            .field("infix", &self.infix)
            .field("suffix", &self.suffix)
            .field("format", &self.format)
            .field("compression", &self.compression)
            //       .field("compression_level", &self.compression_level)
            //.field("collected", &self.collected)
            .field("ix_separator", &self.ix_separator)
            .finish()
    }
}

impl Clone for Inspect {
    fn clone(&self) -> Self {
        Inspect {
            n: self.n,
            segment: self.segment.clone(),
            segment_index: self.segment_index.clone(),
            infix: self.infix.clone(),
            suffix: self.suffix.clone(),
            format: self.format,
            compression: self.compression,
            compression_level: self.compression_level,
            collector: Vec::new(), // do not clone collected data
            collected: 0,
            ix_separator: self.ix_separator.clone(),
            writer: None, // do not clone writer
        }
    }
}

impl Step for Inspect {
    fn move_inited(&mut self) -> Self {
        let mut new = self.clone();
        new.writer = self.writer.take();
        new.collector = self.collector.drain(..).collect();
        new
    }

    fn needs_serial(&self) -> bool {
        true
    }
    fn validate_others(
        &self,
        _input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        // Validate compression level
        crate::config::validate_compression_level_u8(self.compression, self.compression_level)?;
        if !matches!(self.format, FileFormat::Fastq | FileFormat::Fasta) {
            bail!(
                "Inspect step supports only 'fastq' or 'fasta' formats. Received: {:?}",
                self.format
            );
        }
        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        let selection = self.segment.validate(input_def)?;
        self.segment_index = Some(selection);
        Ok(())
    }

    fn configure_output_separator(&mut self, ix_separator: &str) {
        self.ix_separator = ix_separator.to_string();
    }

    fn init(
        &mut self,
        input_info: &crate::transformations::InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplex,
        allow_overwrite: bool,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        self.collector = match self.segment_index.unwrap() {
            SegmentIndexOrAll::All => (0..input_info.segment_order.len())
                .map(|_| Vec::with_capacity(self.n))
                .collect(),
            SegmentIndexOrAll::Indexed(_) => vec![Vec::with_capacity(self.n)],
        };
        self.collected = 0;
        let format_suffix = FileFormat::Fastq.get_suffix(self.compression, self.suffix.as_ref());

        let target = match self.segment_index.unwrap() {
            SegmentIndexOrAll::Indexed(idx) => input_info.segment_order[idx].clone(),
            SegmentIndexOrAll::All => "interleaved".to_string(),
        };

        let base = crate::join_nonempty(
            [output_prefix, self.infix.as_str(), target.as_str()],
            &self.ix_separator,
        );

        let full_path = output_directory.join(format!("{base}.{format_suffix}"));
        crate::output::ensure_output_destination_available(&full_path, allow_overwrite)?;

        let report_file = ex::fs::File::create(full_path)?;
        self.writer = Some(HashedAndCompressedWriter::new(
            report_file,
            self.compression,
            false, // hash_uncompressed
            false, // hash_compressed
            self.compression_level,
            None,
        )?);
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplex,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if self.collected >= self.n {
            return Ok((block, true));
        }

        let mut iter = block.get_pseudo_iter();
        while let Some(read) = iter.pseudo_next() {
            if self.collected >= self.n {
                break;
            }

            match self.segment_index.unwrap() {
                SegmentIndexOrAll::All => {
                    for (collector_idx, segment_index) in
                        (0..input_info.segment_order.len()).enumerate()
                    {
                        let segment_read = &read.segments[segment_index];
                        self.collector[collector_idx].push((
                            segment_read.name().to_vec(),
                            segment_read.seq().to_vec(),
                            segment_read.qual().to_vec(),
                        ));
                    }
                }
                SegmentIndexOrAll::Indexed(idx) => {
                    let segment_read = &read.segments[idx];
                    self.collector[0].push((
                        segment_read.name().to_vec(),
                        segment_read.seq().to_vec(),
                        segment_read.qual().to_vec(),
                    ));
                }
            }

            self.collected += 1; //count per molecule, not per segment
        }
        Ok((block, true))
    }
    fn finalize(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplex,
    ) -> Result<Option<FinalizeReportResult>> {
        // Build filename with format-specific suffix
        let mut writer = self.writer.take().unwrap();
        if !self.collector.is_empty() {
            let reads_to_write = self.collected.min(self.n);
            for read_idx in 0..reads_to_write {
                for segment_reads in &self.collector {
                    if let Some((name, seq, qual)) = segment_reads.get(read_idx) {
                        writer.write_all(b"@")?;
                        writer.write_all(name)?;
                        writer.write_all(b"\n")?;
                        writer.write_all(seq)?;
                        writer.write_all(b"\n+\n")?;
                        writer.write_all(qual)?;
                        writer.write_all(b"\n")?;
                    }
                }
            }
        }

        writer.finish();
        Ok(None)
    }
}
