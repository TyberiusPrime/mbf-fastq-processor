use super::Parser;
use crate::io::{FastQBlock, FastQElement, FastQRead};
use anyhow::{Context, Result};
use bio::io::fasta::{self, FastaRead, Record as FastaRecord};
use ex::fs::File;
use niffler;
use std::io::{BufReader, Read};

type BoxedFastaReader = fasta::Reader<BufReader<Box<dyn Read + Send>>>;

pub struct FastaParser {
    files: Vec<File>,
    current_reader: Option<BoxedFastaReader>,
    target_reads_per_block: usize,
    fake_quality_char: u8,
}

impl FastaParser {
    pub fn new(
        mut files: Vec<File>,
        target_reads_per_block: usize,
        fake_quality_phred: u8,
    ) -> Result<FastaParser> {
        files.reverse();
        let fake_quality_char = fake_quality_phred;
        Ok(FastaParser {
            files,
            current_reader: None,
            target_reads_per_block,
            fake_quality_char,
        })
    }

    fn ensure_reader(&mut self) -> Result<bool> {
        if self.current_reader.is_some() {
            return Ok(true);
        }
        match self.files.pop() {
            Some(file) => {
                let (reader, _format) = niffler::send::get_reader(Box::new(file))?;
                let buffered = BufReader::new(reader);
                self.current_reader = Some(fasta::Reader::from_bufread(buffered));
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

impl Parser for FastaParser {
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

            let reader = self
                .current_reader
                .as_mut()
                .expect("reader must exist after ensure_reader");

            let mut record = FastaRecord::new();
            reader.read(&mut record)?;
            if record.is_empty() {
                self.current_reader = None;
                if block.entries.is_empty() {
                    if self.files.is_empty() {
                        return Ok((block, true));
                    }
                    continue;
                }
                let finished = self.files.is_empty();
                return Ok((block, finished));
            }

            let mut name = record.id().as_bytes().to_vec();
            if let Some(desc) = record.desc() {
                if !desc.is_empty() {
                    name.push(b' ');
                    name.extend_from_slice(desc.as_bytes());
                }
            }
            let seq = record.seq().to_vec();
            let qual = vec![self.fake_quality_char; seq.len()];
            let read = FastQRead::new(
                FastQElement::Owned(name),
                FastQElement::Owned(seq),
                FastQElement::Owned(qual),
            )
            .with_context(|| "Failed to convert FASTA record into synthetic FASTQ read")?;
            block.entries.push(read);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[allow(clippy::match_wildcard_for_single_variants)]
    #[test]
    fn parses_fasta_records_into_fastq_reads() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1\nACGT\n>read2 description\nTGCA\n")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let mut reads = block.entries.into_iter();
        let first = reads.next().expect("test should have expected number of reads");
        match first.name {
            FastQElement::Owned(name) => assert_eq!(name, b"read1".to_vec()),
            _ => panic!("expected owned name"),
        }
        match first.seq {
            FastQElement::Owned(seq) => assert_eq!(seq, b"ACGT".to_vec()),
            _ => panic!("expected owned sequence"),
        }
        match first.qual {
            FastQElement::Owned(qual) => assert_eq!(qual, vec![30; 4]),
            _ => panic!("expected owned qualities"),
        }

        let second = reads.next().expect("test should have expected number of reads");
        match second.name {
            FastQElement::Owned(name) => assert_eq!(name, b"read2 description".to_vec()),
            _ => panic!("expected owned name"),
        }
        match second.seq {
            FastQElement::Owned(seq) => assert_eq!(seq, b"TGCA".to_vec()),
            _ => panic!("expected owned sequence"),
        }
        match second.qual {
            FastQElement::Owned(qual) => assert_eq!(qual, vec![30; 4]),
            _ => panic!("expected owned qualities"),
        }

        let (second_block, is_final) = parser.parse()?;
        assert!(is_final);
        assert!(second_block.entries.is_empty());

        Ok(())
    }
}
