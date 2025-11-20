use std::path::Path;

use super::{ParseResult, Parser};
use crate::io::{FastQBlock, FastQElement, FastQRead};
use anyhow::{Context, Result};
use bstr::ByteSlice;
use ex::fs::File;
use noodles::bam::{self, record::Record};
use noodles::bgzf;

use noodles::bam::bai;
use noodles::csi::binning_index::{BinningIndex, ReferenceSequence};

type BamReader = bam::io::Reader<bgzf::io::Reader<File>>;

pub struct BamParser {
    reader: BamReader,
    target_reads_per_block: usize,
    include_mapped: bool,
    include_unmapped: bool,
    record: Record,
}

pub fn bam_reads_from_index(
    filename: impl AsRef<Path>,
    include_mapped: bool,
    include_unmapped: bool,
) -> Option<usize> {
    let path = filename.as_ref();
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("bam"))
    {
        let candidates = [
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bam.bai");
                idx
            },
            {
                let mut idx = path.to_path_buf();
                idx.set_extension("bai");
                idx
            },
        ];

        for index_path in candidates {
            if !index_path.exists() {
                continue;
            }

            match bai::fs::read(&index_path) {
                Ok(index) => {
                    let total_reads: u128 = index
                        .reference_sequences()
                        .iter()
                        .filter_map(|reference| reference.metadata())
                        .map(|metadata| {
                            let mut total = 0u128;
                            if include_mapped {
                                total += u128::from(metadata.mapped_record_count());
                            }
                            if include_unmapped {
                                total += u128::from(metadata.unmapped_record_count());
                            }
                            total
                        })
                        .sum::<u128>()
                        + (if include_unmapped {
                            u128::from(index.unplaced_unmapped_record_count().unwrap_or(0))
                        } else {
                            0
                        });

                    if total_reads > 0 {
                        return Some(total_reads.try_into().expect("Read count exceeded usize"));
                    }
                    return None;
                }
                Err(error) => {
                    log::debug!(
                        "Failed to read BAM index {index_path:?} for {:?}: {error}",
                        path.display()
                    );
                }
            }
        }
    }
    return None;
}

impl BamParser {
    pub fn new(
        file: File,
        target_reads_per_block: usize,
        include_mapped: bool,
        include_unmapped: bool,
    ) -> Result<BamParser> {
        let mut reader = bam::io::reader::Builder.build_from_reader(file);
        reader.read_header()?;

        Ok(BamParser {
            reader,
            target_reads_per_block,
            include_mapped,
            include_unmapped,
            record: Record::default(),
        })
    }

    fn should_yield_record(&self, record: &Record) -> bool {
        let is_mapped = record.reference_sequence_id().is_some();
        (is_mapped && self.include_mapped) || (!is_mapped && self.include_unmapped)
    }
}

impl Parser for BamParser {
    fn bytes_per_base(&self) -> f64 {
        1.0 // about right
    }

    fn parse(&mut self) -> Result<ParseResult> {
        let mut block = FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
        };

        loop {
            if block.entries.len() >= self.target_reads_per_block {
                return Ok(ParseResult {
                    fastq_block: block,
                    was_final: false,
                });
            }

            let state = &mut self.reader;

            self.record = Record::default();
            match state.read_record(&mut self.record)? {
                0 => {
                    //nothing read.
                    return Ok(ParseResult {
                        fastq_block: block,
                        was_final: true,
                    });
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
        let mut parser = BamParser::new(file, 10, true, false)?;
        let ParseResult {
            fastq_block: block,
            was_final: finished,
        } = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 1);
        if let FastQElement::Owned(name) = &block.entries[0].name {
            assert_eq!(name, b"mapped");
        } else {
            panic!("expected owned name");
        }

        let file = open(temp.path())?;
        let mut parser = BamParser::new(file, 10, false, true)?;
        let ParseResult {
            fastq_block: block,
            was_final: finished,
        } = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 1);
        if let FastQElement::Owned(name) = &block.entries[0].name {
            assert_eq!(name, b"unmapped");
        } else {
            panic!("expected owned name");
        }

        let file = open(temp.path())?;
        let mut parser = BamParser::new(file, 10, true, true)?;
        let ParseResult {
            fastq_block: block,
            was_final: finished,
        } = parser.parse()?;
        assert!(finished);
        assert_eq!(block.entries.len(), 2);

        Ok(())
    }
}
