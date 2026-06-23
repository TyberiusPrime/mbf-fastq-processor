use anyhow::Result;
use bio::io::fasta::{self, FastaRead, Record as FastaRecord};
use ex::fs::File;
use niffler;
use std::{
    io::{BufReader, Read},
    num::NonZero,
    path::PathBuf,
};

use crate::blocks::FastQChunk;
use crate::io::input::{DecompressionOptions, spawn_rapidgzip};
use crate::io::parsers::{ParseResult, Parser, ParserOutput};
use stringpod::{DualStringPodBuilder, StringPod, StringPodBuilder};

type BoxedFastaReader = fasta::Reader<BufReader<Box<dyn Read + Send>>>;

pub struct FastaParser {
    reader: BoxedFastaReader,
    target_reads_per_block: NonZero<usize>,
    fake_quality_char: u8,
    compression_format: niffler::send::compression::Format,
}

impl FastaParser {
    /// # Panics
    /// when rapidgzip & stdin are specified together (validation prevents this)
    pub fn new(
        file: File,
        filename: Option<&PathBuf>,
        target_reads_per_block: NonZero<usize>,
        fake_quality_phred: u8,
        decompression_options: DecompressionOptions,
    ) -> Result<FastaParser> {
        let fake_quality_char = fake_quality_phred;

        let (mut reader, format) = niffler::send::get_reader(Box::new(file))?;

        if let DecompressionOptions::Rapidgzip { thread_count } = decompression_options
            && format == niffler::send::compression::Format::Gzip
        {
            let file = spawn_rapidgzip(
                filename
                    .as_ref()
                    .expect("rapid gzip and stdin not supported"),
                thread_count,
            )?; // cov:excl-line
            reader = Box::new(file);
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
        let target: usize = self.target_reads_per_block.into();
        let mut names = StringPodBuilder::with_capacity(0, target);
        let mut seq_quals = DualStringPodBuilder::with_capacity(0, target);
        let mut qual = vec![self.fake_quality_char; 100];
        let mut count = 0usize;
        let mut was_final = false;

        while count < target {
            let mut record = FastaRecord::new();
            self.reader.read(&mut record)?;
            if record.is_empty() {
                was_final = true;
                break;
            }

            let seq = record.seq();
            if qual.len() < seq.len() {
                //mutant false positive, <= isn't harmful, just tad slower
                qual.resize(seq.len(), self.fake_quality_char);
            }

            // FASTA carries no '+'/quality, so qualities are faked and the plus
            // column is left empty (filled in one shot below).
            match record.desc() {
                Some(desc) => {
                    let id = record.id().as_bytes();
                    let desc = desc.as_bytes();
                    let mut name = Vec::with_capacity(id.len() + 1 + desc.len());
                    name.extend_from_slice(id);
                    name.push(b' ');
                    name.extend_from_slice(desc);
                    names.push(name.as_slice());
                }
                _ => names.push(record.id().as_bytes()),
            }
            seq_quals.push(seq, &qual[..seq.len()]);
            count += 1;
        }

        Ok(ParseResult {
            output: ParserOutput::Chunk(FastQChunk {
                names: names.finish(),
                seq_quals: seq_quals.finish(),
                pluses: StringPod::new_all_empty(
                    u32::try_from(count).expect("too many reads in a block for u32"),
                ),
            }),
            was_final,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use bstr::ByteSlice;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_fasta_records_into_fastq_reads() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1\nACGT\n>read2 description\nTGCA\n")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(
            file,
            Some(temp.path().to_owned()).as_ref(),
            NonZero::new(10).unwrap(),
            30,
            DecompressionOptions::Default,
        )?; // cov:excl-line

        let ParseResult { output, was_final } = parser.parse()?;
        let chunk = output.into_chunk();
        assert!(was_final);
        assert_eq!(chunk.len(), 2);

        assert_eq!(chunk.names.get(0).as_bytes(), b"read1");
        let (seq0, qual0) = chunk.seq_quals.pair(0);
        assert_eq!(seq0.as_bytes(), b"ACGT");
        assert_eq!(qual0.as_bytes(), [30u8; 4].as_slice());
        // FASTA has no '+' line — the plus column is empty for every read.
        assert_eq!(chunk.pluses.get(0).as_bytes(), b"");

        assert_eq!(chunk.names.get(1).as_bytes(), b"read2 description");
        let (seq1, qual1) = chunk.seq_quals.pair(1);
        assert_eq!(seq1.as_bytes(), b"TGCA");
        assert_eq!(qual1.as_bytes(), [30u8; 4].as_slice());

        let ParseResult {
            output: second_output,
            was_final: is_final,
        } = parser.parse()?;
        let second_chunk = second_output.into_chunk();

        assert!(is_final);
        assert!(second_chunk.is_empty());

        Ok(())
    }
}
