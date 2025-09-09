use super::super::{FinalizeReportResult, Step, Target, Transformation, validate_target};
use crate::demultiplex::Demultiplexed;
use crate::output::HashedAndCompressedWriter;
use anyhow::Result;
use std::{io::Write, path::Path};

pub type NameSeqQualTuple = (Vec<u8>, Vec<u8>, Vec<u8>);

#[derive(eserde::Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Inspect {
    pub n: usize,
    pub target: Target,
    pub infix: String,
    #[serde(default)]
    pub suffix: Option<String>,
    #[serde(default)]
    pub format: crate::config::FileFormat,
    #[serde(default)]
    pub compression_level: Option<u8>,
    #[serde(skip)]
    pub collector: Vec<NameSeqQualTuple>,
}

impl Step for Inspect {
    fn needs_serial(&self) -> bool {
        true
    }
    fn validate(
        &self,
        input_def: &crate::config::Input,
        _output_def: Option<&crate::config::Output>,
        _all_transforms: &[Transformation],
        _this_transforms_index: usize,
    ) -> Result<()> {
        validate_target(self.target, input_def)?;

        // Validate compression level
        if let Err(e) =
            crate::config::validate_compression_level_u8(self.format, self.compression_level)
        {
            return Err(anyhow::anyhow!("{}", e));
        }

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
        _block_no: usize,
        _demultiplex_info: &Demultiplexed,
    ) -> (crate::io::FastQBlocksCombined, bool) {
        let collector = &mut self.collector;
        let source = match self.target {
            Target::Read1 => &block.read1,
            Target::Read2 => block.read2.as_ref().unwrap(),
            Target::Index1 => block.index1.as_ref().unwrap(),
            Target::Index2 => block.index2.as_ref().unwrap(),
        };
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
        (block, true)
    }
    fn finalize(
        &mut self,
        output_prefix: &str,
        output_directory: &Path,
        _demultiplex_info: &Demultiplexed,
    ) -> Result<Option<FinalizeReportResult>> {
        let target = match self.target {
            Target::Read1 => "1",
            Target::Read2 => "2",
            Target::Index1 => "i1",
            Target::Index2 => "i2",
        };

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
