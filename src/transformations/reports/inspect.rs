use super::super::{FinalizeReportResult, Step, Transformation};
use crate::config::{CompressionFormat, FileFormat, SegmentIndex, SegmentIndexOrAll, SegmentOrAll};
use crate::demultiplex::Demultiplexed;
use crate::output::HashedAndCompressedWriter;
use anyhow::{Result, bail};
use std::{io::Write, path::Path};

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>);

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub n: usize,
    #[serde(default)]
    segment: SegmentOrAll,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndexOrAll>, // needed to produce output filename
    #[serde(default)]
    #[serde(skip)]
    selected_segments: Vec<SegmentIndex>,

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
}

impl Step for Inspect {
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
        self.selected_segments = match selection {
            SegmentIndexOrAll::Indexed(idx) => vec![SegmentIndex(idx)],
            SegmentIndexOrAll::All => (0..input_def.segment_count()).map(SegmentIndex).collect(),
        };
        self.segment_index = Some(selection);
        Ok(())
    }

    fn configure_output_separator(&mut self, ix_separator: &str) {
        self.ix_separator = ix_separator.to_string();
    }

    fn init(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        if self.collector.is_empty() {
            self.collector = self
                .selected_segments
                .iter()
                .map(|_| Vec::with_capacity(self.n))
                .collect();
        }
        self.collected = 0;
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        if self.selected_segments.is_empty() || self.collected >= self.n {
            return Ok((block, true));
        }

        if self.collector.is_empty() {
            self.collector = self
                .selected_segments
                .iter()
                .map(|_| Vec::with_capacity(self.n))
                .collect();
        }

        let mut iter = block.get_pseudo_iter();
        while let Some(read) = iter.pseudo_next() {
            if self.collected >= self.n {
                break;
            }

            for (collector_idx, segment_index) in self.selected_segments.iter().enumerate() {
                let segment_read = &read.segments[segment_index.get_index()];
                self.collector[collector_idx].push((
                    segment_read.name().to_vec(),
                    segment_read.seq().to_vec(),
                    segment_read.qual().to_vec(),
                ));
            }

            self.collected += 1; //count per molecule, not per segment
        }
        Ok((block, true))
    }
    fn finalize(
        &mut self,
        input_info: &crate::transformations::InputInfo,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let segment_selection = self
            .segment_index
            .as_ref()
            .expect("segment selection validated earlier");
        let target = match segment_selection {
            SegmentIndexOrAll::Indexed(idx) => input_info.segment_order[*idx].clone(),
            SegmentIndexOrAll::All => "interleaved".to_string(),
        };
        // Build filename with format-specific suffix
        let format_suffix = FileFormat::Fastq.get_suffix(self.compression, self.suffix.as_ref());
        let base = crate::join_nonempty(
            [output_prefix, self.infix.as_str(), target.as_str()],
            &self.ix_separator,
        );

        let report_file =
            ex::fs::File::create(output_directory.join(format!("{base}.{format_suffix}")))?;
        let mut compressed_writer = HashedAndCompressedWriter::new(
            report_file,
            self.compression,
            false, // hash_uncompressed
            false, // hash_compressed
            self.compression_level,
            None,
        )?;

        if !self.collector.is_empty() {
            let reads_to_write = self.collected.min(self.n);
            for read_idx in 0..reads_to_write {
                for segment_reads in &self.collector {
                    if let Some((name, seq, qual)) = segment_reads.get(read_idx) {
                        compressed_writer.write_all(b"@")?;
                        compressed_writer.write_all(name)?;
                        compressed_writer.write_all(b"\n")?;
                        compressed_writer.write_all(seq)?;
                        compressed_writer.write_all(b"\n+\n")?;
                        compressed_writer.write_all(qual)?;
                        compressed_writer.write_all(b"\n")?;
                    }
                }
            }
        }

        compressed_writer.finish();
        Ok(None)
    }
}
