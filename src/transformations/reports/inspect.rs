use super::super::{FinalizeReportResult, Step, Transformation};
use crate::config::SegmentIndex;
use crate::output::HashedAndCompressedWriter;
use crate::{config::Segment, demultiplex::Demultiplexed};
use anyhow::Result;
use std::{io::Write, path::Path};

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>);

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub n: usize,
    #[serde(default)]
    segment: Segment,
    #[serde(default)]
    #[serde(skip)]
    segment_index: Option<SegmentIndex>,

    pub infix: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: crate::config::FileFormat,
    #[serde(default)]
    pub compression_level: Option<u8>,
    #[serde(default)] // eserde compatibility https://github.com/mainmatter/eserde/issues/39
    #[serde(skip)]
    pub collector: Vec<NameSeqQualTuple>,
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
        if let Err(e) =
            crate::config::validate_compression_level_u8(self.format, self.compression_level)
        {
            return Err(anyhow::anyhow!("{}", e));
        }

        Ok(())
    }

    fn validate_segments(&mut self, input_def: &crate::config::Input) -> Result<()> {
        self.segment_index = Some(self.segment.validate(input_def)?);
        Ok(())
    }

    fn init(
        &mut self,
        _input_info: &crate::transformations::InputInfo,
        _output_prefix: &str,
        _output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<crate::demultiplex::DemultiplexInfo>> {
        Ok(None)
    }

    fn apply(
        &mut self,
        block: crate::io::FastQBlocksCombined,
        _input_info: &crate::transformations::InputInfo,
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> anyhow::Result<(crate::io::FastQBlocksCombined, bool)> {
        let collector = &mut self.collector;
        let source = &block.segments[self.segment_index.as_ref().unwrap().get_index()];
        if collector.len() < self.n {
            let mut iter = source.get_pseudo_iter();
            while let Some(read) = iter.pseudo_next() {
                if collector.len() >= self.n {
                    break;
                }
                collector.push((
                    read.name().to_vec(),
                    read.seq().to_vec(),
                    read.qual().to_vec(),
                ));
            }
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
        let target = &input_info.segment_order[self.segment_index.as_ref().unwrap().get_index()];
        // Build filename with format-specific suffix
        let format_suffix = self.format.get_suffix(None);
        let base_filename = format!(
            "{output_prefix}_{infix}_{target}.{format_suffix}",
            infix = self.infix
        );

        let report_file = std::fs::File::create(output_directory.join(&base_filename))?;
        let mut compressed_writer = HashedAndCompressedWriter::new(
            report_file,
            self.format,
            false, // hash_uncompressed
            false, // hash_compressed
            self.compression_level,
        )?;

        for (name, seq, qual) in &self.collector {
            compressed_writer.write_all(b"@")?;
            compressed_writer.write_all(name)?;
            compressed_writer.write_all(b"\n")?;
            compressed_writer.write_all(seq)?;
            compressed_writer.write_all(b"\n+\n")?;
            compressed_writer.write_all(qual)?;
            compressed_writer.write_all(b"\n")?;
        }

        compressed_writer.finish();
        Ok(None)
    }
}
