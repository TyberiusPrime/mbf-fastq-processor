use super::{ParseResult, Parser};
use crate::io::{
    FastQBlock, FastQRead,
    input::{DecompressionOptions, spawn_rapidgzip},
};
use anyhow::{Context, Result};
use bio::io::fasta::{self, FastaRead, Record as FastaRecord};
use ex::fs::File;
use niffler;
use std::{
    io::{BufReader, Read},
    path::PathBuf,
};

type BoxedFastaReader = fasta::Reader<BufReader<Box<dyn Read + Send>>>;

pub struct FastaParser {
    reader: BoxedFastaReader,
    target_reads_per_block: usize,
    fake_quality_char: u8,
    compression_format: niffler::send::compression::Format,
}

impl FastaParser {
    pub fn new(
        file: File,
        filename: Option<&PathBuf>,
        target_reads_per_block: usize,
        fake_quality_phred: u8,
        decompression_options: DecompressionOptions,
    ) -> Result<FastaParser> {
        let fake_quality_char = fake_quality_phred;

        let (mut reader, format) = niffler::send::get_reader(Box::new(file))?;

        if let DecompressionOptions::Rapidgzip {
            thread_count,
            index_gzip,
        } = decompression_options
        {
            // only do rapidgzip if we have more than 2 threads..
            // otherwise, plain gzip decompression is going to be faster
            // since it's optimized better
            if format == niffler::send::compression::Format::Gzip {
                let file = spawn_rapidgzip(
                    filename
                        .as_ref()
                        .expect("rapid gzip and stdin not supported"),
                    thread_count,
                    index_gzip,
                )?;
                reader = Box::new(file);
            }
        }

        let buffered = BufReader::new(reader);
        let reader = fasta::Reader::from_bufread(buffered);
        Ok(FastaParser {
            reader,
            target_reads_per_block,
            fake_quality_char,
            compression_format: format,
        })
    }
}

impl Parser for FastaParser {
    fn bytes_per_base(&self) -> f64 {
        match self.compression_format {
            niffler::send::compression::Format::Gzip
            | niffler::send::compression::Format::Bzip
            | niffler::send::compression::Format::Lzma
            | niffler::send::compression::Format::Zstd => 0.38,
            niffler::send::compression::Format::No => 1.4,
        }
    }
    fn parse(&mut self) -> Result<ParseResult> {
        let mut block = FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
        };
        let mut qual = vec![self.fake_quality_char; 100];

        loop {
            if block.entries.len() >= self.target_reads_per_block {
                return Ok(ParseResult {
                    fastq_block: block,
                    was_final: false,
                });
            }

            let reader = &mut self.reader;

            let mut record = FastaRecord::new();
            reader.read(&mut record)?;
            if record.is_empty() {
                return Ok(ParseResult {
                    fastq_block: block,
                    was_final: true,
                });
            }

            let (combined_iter, combined_len): (Box<dyn Iterator<Item = u8>>, usize) =
                match record.desc() {
                    Some(desc) => {
                        let desc_bytes = desc.as_bytes();
                        let name_bytes = record.id().as_bytes();
                        let name_len = name_bytes.len();
                        let desc_iter = b" ".iter().chain(desc_bytes.iter());
                        (
                            Box::new(name_bytes.iter().chain(desc_iter).copied()),
                            name_len + 1 + desc_bytes.len(),
                        )
                    }
                    _ => (
                        Box::new(record.id().as_bytes().iter().copied()),
                        record.id().len(),
                    ),
                };
            let seq = record.seq();
            if qual.len() < seq.len() {
                qual.resize(seq.len(), self.fake_quality_char);
            }
            let read = FastQRead::new(
                block.append_element_from_iter(combined_iter, combined_len),
                block.append_element(record.seq()),
                block.append_element(&qual[..seq.len()]),
            )
            .with_context(|| "Failed to convert FASTA record into synthetic FASTQ read")?;
            block.entries.push(read);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::io::FastQElement;

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
        let mut parser = FastaParser::new(
            file,
            Some(temp.path().to_owned()).as_ref(),
            10,
            30,
            DecompressionOptions::Default,
        )?;

        let ParseResult {
            fastq_block: block,
            was_final,
        } = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let mut reads = block.entries.into_iter();
        let first = reads
            .next()
            .expect("test should have expected number of reads");
        match first.name {
            FastQElement::Local(_) => assert_eq!(first.name.get(&block.block), b"read1".to_vec()),
            _ => panic!("expected Local name"),
        }
        match first.seq {
            FastQElement::Local(_) => assert_eq!(first.seq.get(&block.block), b"ACGT".to_vec()),
            _ => panic!("expected Local sequence"),
        }
        match first.qual {
            FastQElement::Local(_) => assert_eq!(first.qual.get(&block.block), vec![30; 4]),
            _ => panic!("expected Local qualities"),
        }

        let second = reads
            .next()
            .expect("test should have expected number of reads");
        match second.name {
            FastQElement::Local(_) => {
                assert_eq!(second.name.get(&block.block), b"read2 description".to_vec());
            }
            _ => panic!("expected Local name"),
        }
        match second.seq {
            FastQElement::Local(_) => assert_eq!(second.seq.get(&block.block), b"TGCA".to_vec()),
            _ => panic!("expected Local sequence"),
        }
        match second.qual {
            FastQElement::Local(_) => assert_eq!(second.qual.get(&block.block), vec![30; 4]),
            _ => panic!("expected Local qualities"),
        }
        let ParseResult {
            fastq_block: second_block,
            was_final: is_final,
        } = parser.parse()?;

        assert!(is_final);
        assert!(second_block.entries.is_empty());

        Ok(())
    }
}
