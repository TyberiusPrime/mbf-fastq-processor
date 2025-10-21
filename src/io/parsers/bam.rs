use crate::io::{FastQBlock, FastQElement, FastQRead};
use super::Parser;
use anyhow::{Context, Result};
use bstr::ByteSlice;
use ex::fs::File;
use noodles::bam::{self, record::Record};
use noodles::bgzf;

type BamReader = bam::io::Reader<bgzf::io::Reader<File>>;

struct BamState {
    reader: BamReader,
}

pub struct BamParser {
    files: Vec<File>,
    current: Option<BamState>,
    target_reads_per_block: usize,
    include_mapped: bool,
    include_unmapped: bool,
    record: Record,
}

impl BamParser {
    pub fn new(
        mut files: Vec<File>,
        target_reads_per_block: usize,
        include_mapped: bool,
        include_unmapped: bool,
    ) -> Result<BamParser> {
        files.reverse();
        Ok(BamParser {
            files,
            current: None,
            target_reads_per_block,
            include_mapped,
            include_unmapped,
            record: Record::default(),
        })
    }

    fn ensure_reader(&mut self) -> Result<bool> {
        if self.current.is_some() {
            return Ok(true);
        }
        match self.files.pop() {
            Some(file) => {
                let mut reader = bam::io::reader::Builder::default().build_from_reader(file);
                reader.read_header()?;
                self.current = Some(BamState { reader });
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn should_yield_record(&self, record: &Record) -> bool {
        let is_mapped = record.reference_sequence_id().is_some();
        (is_mapped && self.include_mapped) || (!is_mapped && self.include_unmapped)
    }
}

impl Parser for BamParser {
    fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        let mut block = FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
        };

        loop {
            if block.entries.len() >= self.target_reads_per_block {
                return Ok((block, false));
            }

            if !self.ensure_reader()? {
                return Ok((block, true));
            }

            let state = self
                .current
                .as_mut()
                .expect("reader must exist after ensure_reader");

            self.record = Record::default();
            match state.reader.read_record(&mut self.record)? {
                0 => {
                    self.current = None;
                    if block.entries.is_empty() {
                        if self.files.is_empty() {
                            return Ok((block, true));
                        }
                        continue;
                    }
                    let finished = self.files.is_empty();
                    return Ok((block, finished));
                }
                _ => {
                    if !self.should_yield_record(&self.record) {
                        continue;
                    }
                    let name = self
                        .record
                        .name()
                        .map(|n| n.as_bytes().to_vec())
                        .unwrap_or_default();
                    let seq: Vec<u8> = self.record.sequence().iter().collect();
                    let qual: Vec<u8> = if self.record.quality_scores().is_empty() {
                        vec![b'!'; seq.len()]
                    } else {
                        self.record
                            .quality_scores()
                            .iter()
                            .map(|q| q + 33)
                            .collect()
                    };
                    let read = FastQRead::new(
                        FastQElement::Owned(name),
                        FastQElement::Owned(seq),
                        FastQElement::Owned(qual),
                    )
                    .with_context(|| "Failed to convert BAM record into FastQ-like read")?;
                    block.entries.push(read);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use noodles::bam;
    use noodles::sam::alignment::io::Write;
    use noodles::sam::{
        self,
        alignment::record::Flags as SamFlags,
        alignment::record_buf::{QualityScores as SamQualityScores, Sequence as SamSequence},
        header::record::value::{Map, map::ReferenceSequence},
    };
    use std::num::NonZeroUsize;
    use tempfile::NamedTempFile;

    fn write_test_bam(path: &std::path::Path) -> Result<()> {
        let reference_length = NonZeroUsize::new(100).unwrap();
        let header = sam::Header::builder()
            .add_reference_sequence("chr1", Map::<ReferenceSequence>::new(reference_length))
            .build();

        let file = std::fs::File::create(path)?;
        let mut writer = bam::io::Writer::new(file);
        writer.write_header(&header)?;

        let mut mapped = sam::alignment::RecordBuf::default();
        *mapped.name_mut() = Some("mapped".into());
        *mapped.flags_mut() = SamFlags::empty();
        *mapped.reference_sequence_id_mut() = Some(0);
        *mapped.sequence_mut() = SamSequence::from(b"ACGT".to_vec());
        *mapped.quality_scores_mut() = SamQualityScores::from(vec![30, 30, 30, 30]);
        writer.write_alignment_record(&header, &mapped)?;

        let mut unmapped = sam::alignment::RecordBuf::default();
        *unmapped.name_mut() = Some("unmapped".into());
        *unmapped.flags_mut() = SamFlags::UNMAPPED;
        *unmapped.sequence_mut() = SamSequence::from(b"TGCA".to_vec());
        *unmapped.quality_scores_mut() = SamQualityScores::from(vec![25, 25, 25, 25]);
        writer.write_alignment_record(&header, &unmapped)?;

        writer.try_finish()?;
        Ok(())
    }

    #[test]
    fn respects_mapped_and_unmapped_filters() -> Result<()> {
        let temp = NamedTempFile::new()?;
        write_test_bam(temp.path())?;

        let open = |path: &std::path::Path| -> Result<File> { Ok(File::open(path)?) };

        let file = open(temp.path())?;
        let mut parser = BamParser::new(vec![file], 10, true, false)?;
        let (block, finished) = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 1);
        if let FastQElement::Owned(name) = &block.entries[0].name {
            assert_eq!(name, b"mapped");
        } else {
            panic!("expected owned name");
        }

        let file = open(temp.path())?;
        let mut parser = BamParser::new(vec![file], 10, false, true)?;
        let (block, finished) = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 1);
        if let FastQElement::Owned(name) = &block.entries[0].name {
            assert_eq!(name, b"unmapped");
        } else {
            panic!("expected owned name");
        }

        let file = open(temp.path())?;
        let mut parser = BamParser::new(vec![file], 10, true, true)?;
        let (block, finished) = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 2);

        Ok(())
    }
}
