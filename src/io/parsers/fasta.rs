use super::Parser;
use crate::io::{FastQBlock, FastQElement, FastQRead, Position};
use anyhow::{Context, Result, bail};
use ex::fs::File;
use niffler;
use std::io::Read;

pub struct FastaParser {
    readers: Vec<File>,
    current_reader: Option<Box<dyn Read + Send>>,
    raw_buffer: Vec<u8>,
    raw_pos: usize,
    target_reads_per_block: usize,
    buf_size: usize,
    fake_quality_char: u8,
    finished: bool,
    partial: PartialRecord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordStage {
    Header,
    Sequence,
}

struct PartialRecord {
    name: Vec<u8>,
    seq: Vec<u8>,
    stage: RecordStage,
    line_start: bool,
}

impl Default for PartialRecord {
    fn default() -> Self {
        PartialRecord {
            name: Vec::new(),
            seq: Vec::new(),
            stage: RecordStage::Header,
            line_start: true,
        }
    }
}

impl PartialRecord {
    fn reset(&mut self) {
        self.name.clear();
        self.seq.clear();
        self.stage = RecordStage::Header;
        self.line_start = true;
    }

    fn is_idle(&self) -> bool {
        self.name.is_empty() && self.seq.is_empty() && self.stage == RecordStage::Header
    }
}

impl FastaParser {
    pub fn new(
        mut files: Vec<File>,
        target_reads_per_block: usize,
        buf_size: usize,
        fake_quality_phred: u8,
    ) -> Result<FastaParser> {
        files.reverse();
        Ok(FastaParser {
            readers: files,
            current_reader: None,
            raw_buffer: Vec::new(),
            raw_pos: 0,
            target_reads_per_block,
            buf_size,
            fake_quality_char: fake_quality_phred,
            finished: false,
            partial: PartialRecord::default(),
        })
    }

    fn ensure_reader(&mut self) -> Result<bool> {
        if self.current_reader.is_some() {
            return Ok(true);
        }
        match self.readers.pop() {
            Some(file) => {
                let (reader, _format) = niffler::send::get_reader(Box::new(file))?;
                self.current_reader = Some(reader);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn ensure_data(&mut self) -> Result<bool> {
        if self.raw_pos < self.raw_buffer.len() {
            return Ok(true);
        }
        self.raw_buffer.clear();
        self.raw_pos = 0;
        while !self.finished {
            if self.ensure_reader()? {
                if let Some(reader) = self.current_reader.as_mut() {
                    self.raw_buffer.resize(self.buf_size, 0);
                    let read = reader.read(&mut self.raw_buffer)?;
                    if read == 0 {
                        self.current_reader = None;
                        continue;
                    }
                    self.raw_buffer.truncate(read);
                    return Ok(true);
                }
            } else {
                self.finished = true;
            }
        }
        Ok(false)
    }

    fn peek_byte(&mut self) -> Result<Option<u8>> {
        if self.ensure_data()? {
            Ok(Some(self.raw_buffer[self.raw_pos]))
        } else {
            Ok(None)
        }
    }

    fn next_byte(&mut self) -> Result<Option<u8>> {
        if self.ensure_data()? {
            let b = self.raw_buffer[self.raw_pos];
            self.raw_pos += 1;
            Ok(Some(b))
        } else {
            Ok(None)
        }
    }

    fn skip_optional_lf(&mut self) -> Result<()> {
        if matches!(self.peek_byte()?, Some(b'\n')) {
            self.raw_pos += 1;
        }
        Ok(())
    }

    fn read_header(&mut self) -> Result<bool> {
        if self.partial.stage != RecordStage::Header {
            return Ok(true);
        }

        loop {
            match self.peek_byte()? {
                Some(b'>') => {
                    self.raw_pos += 1;
                    break;
                }
                Some(b'\n') | Some(b'\r') | Some(b' ' | b'\t') => {
                    self.raw_pos += 1;
                }
                Some(other) => {
                    bail!("Expected '>' at start of FASTA record, found byte 0x{other:02x}");
                }
                None => {
                    self.finished = true;
                    return Ok(false);
                }
            }
        }

        self.partial.name.clear();
        loop {
            match self.next_byte()? {
                Some(b'\n') => {
                    self.partial.stage = RecordStage::Sequence;
                    self.partial.line_start = true;
                    return Ok(true);
                }
                Some(b'\r') => {
                    self.skip_optional_lf()?;
                    self.partial.stage = RecordStage::Sequence;
                    self.partial.line_start = true;
                    return Ok(true);
                }
                Some(byte) => self.partial.name.push(byte),
                None => {
                    self.finished = true;
                    if self.partial.name.is_empty() {
                        bail!("Incomplete FASTA header at end of file");
                    }
                    self.partial.stage = RecordStage::Sequence;
                    self.partial.line_start = true;
                    return Ok(true);
                }
            }
        }
    }

    fn emit_record(&mut self, block: &mut FastQBlock) -> Result<()> {
        if self.partial.name.is_empty() {
            bail!("Encountered FASTA sequence without a header line");
        }
        let name_start = block.block.len();
        block.block.extend_from_slice(&self.partial.name);
        let name_end = block.block.len();
        let seq_start = block.block.len();
        block.block.extend_from_slice(&self.partial.seq);
        let seq_end = block.block.len();
        let qual = vec![self.fake_quality_char; seq_end - seq_start];
        let read = FastQRead::new(
            FastQElement::Local(Position {
                start: name_start,
                end: name_end,
            }),
            FastQElement::Local(Position {
                start: seq_start,
                end: seq_end,
            }),
            FastQElement::Owned(qual),
        )
        .with_context(|| {
            format!(
                "Failed to materialize FASTA record '{}'",
                String::from_utf8_lossy(&self.partial.name)
            )
        })?;
        block.entries.push(read);
        self.partial.reset();
        Ok(())
    }

    fn read_sequence(&mut self, block: &mut FastQBlock) -> Result<bool> {
        if self.partial.stage != RecordStage::Sequence {
            return Ok(false);
        }

        loop {
            match self.peek_byte()? {
                Some(b'>') if self.partial.line_start => {
                    self.emit_record(block)?;
                    return Ok(true);
                }
                Some(byte) => {
                    self.raw_pos += 1;
                    match byte {
                        b'\n' => {
                            self.partial.line_start = true;
                        }
                        b'\r' => {
                            self.partial.line_start = true;
                            self.skip_optional_lf()?;
                        }
                        b' ' | b'\t' => {}
                        _ => {
                            self.partial.seq.push(byte);
                            self.partial.line_start = false;
                        }
                    }
                }
                None => {
                    if self.partial.name.is_empty() && self.partial.seq.is_empty() {
                        return Ok(false);
                    }
                    self.emit_record(block)?;
                    self.finished = true;
                    return Ok(true);
                }
            }
        }
    }

    fn try_read_record(&mut self, block: &mut FastQBlock) -> Result<bool> {
        if !self.read_header()? {
            return Ok(false);
        }
        if self.read_sequence(block)? {
            return Ok(true);
        }
        Ok(false)
    }
}

impl Parser for FastaParser {
    fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        let mut block = FastQBlock {
            block: Vec::new(),
            entries: Vec::new(),
        };

        while block.entries.len() < self.target_reads_per_block {
            if !self.try_read_record(&mut block)? {
                let is_final = self.finished && self.partial.is_idle();
                return Ok((block, is_final));
            }
        }

        Ok((block, false))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_fasta_records_into_fastq_reads() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1\nACGT\n>read2 description\nTGCA\n")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 8 * 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let mut reads = block.entries.into_iter();
        let first = reads.next().unwrap();
        if let FastQElement::Local(_) = first.name {
            assert_eq!(first.name.get(&block.block), b"read1");
        } else {
            panic!("expected local name");
        }
        if let FastQElement::Local(_) = first.seq {
            assert_eq!(first.seq.get(&block.block), b"ACGT");
        } else {
            panic!("expected local sequence");
        }
        if let FastQElement::Owned(ref qual) = first.qual {
            assert_eq!(qual, &vec![30; 4]);
        } else {
            panic!("expected owned qualities");
        }

        let second = reads.next().unwrap();
        if let FastQElement::Local(_) = second.name {
            assert_eq!(second.name.get(&block.block), b"read2 description");
        } else {
            panic!("expected local name");
        }
        if let FastQElement::Local(_) = second.seq {
            assert_eq!(second.seq.get(&block.block), b"TGCA");
        } else {
            panic!("expected local sequence");
        }
        if let FastQElement::Owned(ref qual) = second.qual {
            assert_eq!(qual, &vec![30; 4]);
        } else {
            panic!("expected owned qualities");
        }

        let (second_block, is_final) = parser.parse()?;
        assert!(is_final);
        assert!(second_block.entries.is_empty());

        Ok(())
    }

    #[test]
    fn parses_wrapped_fasta_sequences_without_newlines() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">wrapped\nACGT\nTGCA\n>another\nNNNNNN\n",)?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 16 * 1024, 40)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let first = &block.entries[0];
        assert_eq!(first.seq.get(&block.block), b"ACGTTGCA");
        assert!(
            first
                .seq
                .get(&block.block)
                .iter()
                .all(|&b| b != b'\n' && b != b'\r')
        );

        let second = &block.entries[1];
        assert_eq!(second.seq.get(&block.block), b"NNNNNN");

        Ok(())
    }
}
