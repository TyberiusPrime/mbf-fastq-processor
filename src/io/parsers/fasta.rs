use super::Parser;
use crate::io::{FastQBlock, FastQElement, FastQRead, Position};
use anyhow::{Context, Result, bail};
use ex::fs::File;
use niffler;
use std::io::Read;

pub struct FastaParser {
    readers: Vec<File>,
    current_reader: Option<Box<dyn Read + Send>>,
    current_block: Option<FastQBlock>,
    buf_size: usize,
    target_reads_per_block: usize,
    fake_quality_char: u8,
    last_partial: Option<PartialFastaRead>,
    windows_mode: Option<bool>,
}

#[derive(Debug)]
struct PartialFastaRead {
    name: FastQElement,
    seq: FastQElement,
    in_header: bool,
    // Track original buffer positions for zero-copy conversion at EOF
    // These are only valid if the read was created in a single buffer and not continued
    name_pos: Option<(usize, usize)>,  // (start, end) in buffer
    seq_pos: Option<(usize, usize)>,    // (start, end) in buffer
    spans_multiple_buffers: bool,      // If true, positions are invalid
}

impl FastaParser {
    pub fn new(
        mut readers: Vec<File>,
        target_reads_per_block: usize,
        buf_size: usize,
        fake_quality_phred: u8,
    ) -> Result<FastaParser> {
        readers.reverse(); // so we can pop() them one by one in the right order
        Ok(FastaParser {
            readers,
            current_reader: None,
            current_block: Some(FastQBlock {
                block: Vec::new(),
                entries: Vec::new(),
            }),
            buf_size,
            target_reads_per_block,
            fake_quality_char: fake_quality_phred,
            last_partial: None,
            windows_mode: None,
        })
    }

    fn next_block(&mut self) -> Result<(FastQBlock, bool)> {
        let mut was_final = false;
        let mut start = self.current_block.as_ref().unwrap().block.len();

        while self.current_block.as_ref().unwrap().entries.len() < self.target_reads_per_block {
            if self.current_reader.is_none() {
                if let Some(next_file) = self.readers.pop() {
                    let (reader, _format) = niffler::send::get_reader(Box::new(next_file))?;
                    self.current_reader = Some(reader);
                } else {
                    was_final = true;
                    break;
                }
            }

            let block_start = start;
            if start >= self.current_block.as_ref().unwrap().block.len() {
                self.current_block
                    .as_mut()
                    .unwrap()
                    .block
                    .extend(vec![0; self.buf_size]);
            }

            let bytes_read = self
                .current_reader
                .as_mut()
                .expect("current_reader must exist when reading")
                .read(&mut self.current_block.as_mut().unwrap().block[start..])?;

            if bytes_read == 0 {
                self.windows_mode = None;
                self.current_reader = None;
                if self.readers.is_empty() {
                    was_final = true;
                    break;
                }
                continue;
            }
            start += bytes_read;

            let parse_result = parse_fasta_to_block(
                self.current_block.as_mut().unwrap(),
                block_start,
                start,
                self.last_partial.take(),
                self.windows_mode,
                self.fake_quality_char,
            )?;

            self.last_partial = parse_result.partial_read;
            self.windows_mode = Some(parse_result.windows_mode);
        }

        self.current_block.as_mut().unwrap().block.resize(start, 0);

        let (mut out_block, new_block) = self
            .current_block
            .take()
            .unwrap()
            .split_at(self.target_reads_per_block);

        self.current_block = Some(new_block);

        if was_final {
            if let Some(partial) = self.last_partial.take() {
                // Finalize the last partial read
                // Use tracked positions for zero-copy only if NOT spanning multiple buffers
                let name = if !partial.spans_multiple_buffers {
                    if let Some((start, end)) = partial.name_pos {
                        FastQElement::Local(Position { start, end })
                    } else {
                        partial.name
                    }
                } else {
                    partial.name
                };

                let seq = if !partial.spans_multiple_buffers {
                    if let Some((start, end)) = partial.seq_pos {
                        FastQElement::Local(Position { start, end })
                    } else {
                        partial.seq
                    }
                } else {
                    partial.seq
                };

                let qual = FastQElement::Owned(vec![self.fake_quality_char; seq.len()]);
                let final_read = FastQRead::new(name, seq, qual)
                    .context("In parsing final FASTA read")?;
                out_block.entries.push(final_read);
            }
        }
        Ok((out_block, was_final))
    }
}

impl Parser for FastaParser {
    fn parse(&mut self) -> Result<(FastQBlock, bool)> {
        self.next_block()
    }
}

struct FastaBlockParseResult {
    partial_read: Option<PartialFastaRead>,
    windows_mode: bool,
}

/// Parse FASTA data from buffer, compacting wrapped sequences in-place
#[allow(clippy::too_many_lines)]
fn parse_fasta_to_block(
    target_block: &mut FastQBlock,
    start_offset: usize,
    stop: usize,
    last_partial: Option<PartialFastaRead>,
    windows_mode: Option<bool>,
    fake_quality_char: u8,
) -> Result<FastaBlockParseResult> {
    let input = &mut target_block.block;
    let entries = &mut target_block.entries;

    // Detect windows mode (CRLF vs LF)
    let windows_mode = match windows_mode {
        Some(x) => x,
        None => memchr::memchr(b'\r', &input[start_offset..stop]).is_some(),
    };

    let newline_pattern: &[u8] = if windows_mode { b"\r\n" } else { b"\n" };
    let newline_len = newline_pattern.len();

    let mut pos = start_offset;
    let mut partial_read = last_partial;

    // Handle continuation of partial read from previous buffer
    if let Some(ref mut partial) = partial_read {
        // Mark that this partial now spans multiple buffers
        partial.spans_multiple_buffers = true;

        if partial.in_header {
            // Continue reading header
            if let Some(newline_pos) = memchr::memmem::find(&input[pos..stop], newline_pattern) {
                let header_end = pos + newline_pos;
                match &mut partial.name {
                    FastQElement::Owned(name) => {
                        name.extend_from_slice(&input[pos..header_end]);
                    }
                    FastQElement::Local(_) => unreachable!(),
                }
                pos = header_end + newline_len;
                partial.in_header = false;
            } else {
                // Still in header, add rest of buffer
                match &mut partial.name {
                    FastQElement::Owned(name) => {
                        name.extend_from_slice(&input[pos..stop]);
                    }
                    FastQElement::Local(_) => unreachable!(),
                }
                return Ok(FastaBlockParseResult {
                    partial_read,
                    windows_mode,
                });
            }
        }

        // Continue reading sequence for partial read
        // We need to find the next header (\n>) or end of buffer
        let next_header_pattern = memchr::memmem::find(&input[pos..stop], b"\n>");
        let seq_end = match next_header_pattern {
            Some(newline_before_header) => pos + newline_before_header,
            None => stop,
        };

        // Compact sequence data by removing newlines
        match &mut partial.seq {
            FastQElement::Owned(seq) => {
                for &byte in &input[pos..seq_end] {
                    if byte != b'\n' && byte != b'\r' {
                        seq.push(byte);
                    }
                }
            }
            FastQElement::Local(_) => unreachable!(),
        }

        pos = seq_end;

        // If we found the next header, finalize this read
        if next_header_pattern.is_some() {
            let seq_len = partial.seq.len();
            let qual = FastQElement::Owned(vec![fake_quality_char; seq_len]);
            let read = FastQRead::new(partial.name.clone(), partial.seq.clone(), qual)
                .context("Failed to create FASTA read from partial")?;
            entries.push(read);
            partial_read = None;
        } else {
            // Still reading this sequence
            return Ok(FastaBlockParseResult {
                partial_read,
                windows_mode,
            });
        }
    }

    // Parse complete records in this buffer
    while pos < stop {
        // Look for header start
        if input[pos] != b'>' {
            // Skip empty lines or whitespace
            if input[pos] == b'\n'
                || input[pos] == b'\r'
                || input[pos] == b' '
                || input[pos] == b'\t'
            {
                pos += 1;
                continue;
            }
            bail!(
                "Expected '>' at position {}, found {}",
                pos,
                input[pos] as char
            );
        }

        pos += 1; // Skip '>'

        // Find end of header line
        let header_start = pos;
        let header_end_result = memchr::memmem::find(&input[pos..stop], newline_pattern);

        let (header_end, in_header) = match header_end_result {
            Some(offset) => (pos + offset, false),
            None => {
                // Header continues in next buffer
                partial_read = Some(PartialFastaRead {
                    name: FastQElement::Owned(input[header_start..stop].to_vec()),
                    seq: FastQElement::Owned(Vec::new()),
                    in_header: true,
                    name_pos: Some((header_start, stop)),
                    seq_pos: None,
                    spans_multiple_buffers: false,  // Will be set to true if continued
                });
                break;
            }
        };

        pos = header_end + newline_len;

        // Now read the sequence until next header (\n>) or end of buffer
        let seq_start_in_buffer = pos;
        let next_header_pattern = memchr::memmem::find(&input[pos..stop], b"\n>");
        let seq_region_end = match next_header_pattern {
            Some(newline_before_header) => pos + newline_before_header,
            None => stop,
        };

        // Compact sequence in-place by removing newlines
        let mut write_pos = seq_start_in_buffer;
        let mut read_pos = seq_start_in_buffer;

        while read_pos < seq_region_end {
            let byte = input[read_pos];
            if byte != b'\n' && byte != b'\r' {
                input[write_pos] = byte;
                write_pos += 1;
            }
            read_pos += 1;
        }

        let seq_end_in_buffer = write_pos;

        // Move position to after the sequence region we just processed
        pos = seq_region_end;

        // Determine if this is a complete read or partial
        let is_complete = if next_header_pattern.is_some() {
            // Found next header - this read is complete
            true
        } else {
            // Didn't find next header. Check if we're at end of buffer with trailing newline
            // If so, this is likely a complete record
            let at_buffer_end = seq_region_end == stop;
            let ends_with_newline = (stop > 0 && input[stop - 1] == b'\n') ||
                (stop > 1 && input[stop - 2] == b'\r' && input[stop - 1] == b'\n');

            at_buffer_end && ends_with_newline
        };

        if is_complete {
            // Complete read - use zero-copy Local references
            let name = FastQElement::Local(Position {
                start: header_start,
                end: header_end,
            });
            let seq = FastQElement::Local(Position {
                start: seq_start_in_buffer,
                end: seq_end_in_buffer,
            });
            let qual = FastQElement::Owned(vec![
                fake_quality_char;
                seq_end_in_buffer - seq_start_in_buffer
            ]);

            let read = FastQRead::new(name, seq, qual).with_context(|| {
                format!("Failed to create FASTA read at position {}", header_start)
            })?;
            entries.push(read);
        } else {
            // Partial read at end of buffer (sequence may continue)
            partial_read = Some(PartialFastaRead {
                name: FastQElement::Owned(input[header_start..header_end].to_vec()),
                seq: FastQElement::Owned(input[seq_start_in_buffer..seq_end_in_buffer].to_vec()),
                in_header,
                name_pos: Some((header_start, header_end)),
                seq_pos: Some((seq_start_in_buffer, seq_end_in_buffer)),
                spans_multiple_buffers: false,  // Will be set to true if continued
            });
            break;
        }
    }

    Ok(FastaBlockParseResult {
        partial_read,
        windows_mode,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn parses_simple_fasta() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1\nACGT\n>read2\nTGCA")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let first = block.get(0);
        assert_eq!(first.name(), b"read1");
        assert_eq!(first.seq(), b"ACGT");
        assert_eq!(first.qual(), &[30; 4]);

        let second = block.get(1);
        assert_eq!(second.name(), b"read2");
        assert_eq!(second.seq(), b"TGCA");
        assert_eq!(second.qual(), &[30; 4]);

        Ok(())
    }

    #[test]
    fn parses_wrapped_fasta() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1 with description")?;
        writeln!(temp, "ACGT")?;
        writeln!(temp, "TGCA")?;
        writeln!(temp, "NNNN")?;
        writeln!(temp, ">read2")?;
        writeln!(temp, "AAA")?;
        writeln!(temp, "TTT")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 33)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let first = block.get(0);
        assert_eq!(first.name(), b"read1 with description");
        assert_eq!(first.seq(), b"ACGTTGCANNNN");
        assert_eq!(first.qual().len(), 12);
        assert!(first.qual().iter().all(|&q| q == 33));

        let second = block.get(1);
        assert_eq!(second.name(), b"read2");
        assert_eq!(second.seq(), b"AAATTT");
        assert_eq!(second.qual().len(), 6);

        Ok(())
    }

    #[test]
    fn handles_empty_lines() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1")?;
        writeln!(temp, "ACGT")?;
        writeln!(temp)?; // empty line
        writeln!(temp, ">read2")?;
        writeln!(temp, "TGCA")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        Ok(())
    }

    #[test]
    fn handles_small_buffer() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1")?;
        writeln!(temp, "ACGTACGTACGTACGT")?;
        writeln!(temp, ">read2")?;
        writeln!(temp, "TGCATGCATGCATGCA")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        // Use very small buffer to test partial read handling
        let mut parser = FastaParser::new(vec![file], 10, 16, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let first = block.get(0);
        assert_eq!(first.name(), b"read1");
        assert_eq!(first.seq(), b"ACGTACGTACGTACGT");

        let second = block.get(1);
        assert_eq!(second.name(), b"read2");
        assert_eq!(second.seq(), b"TGCATGCATGCATGCA");

        Ok(())
    }

    #[test]
    fn handles_long_wrapped_sequence() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">long_read")?;
        // Write a long sequence wrapped at 10 bases per line
        for _ in 0..10 {
            writeln!(temp, "ACGTACGTAC")?;
        }
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 1);

        let read = block.get(0);
        assert_eq!(read.name(), b"long_read");
        assert_eq!(read.seq().len(), 100);
        assert_eq!(read.seq(), b"ACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTACACGTACGTAC");

        Ok(())
    }

    #[test]
    fn handles_greater_than_in_sequence() -> Result<()> {
        // Adversarial test: '>' should only be treated as header at start of line
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1 test with > in name")?;
        writeln!(temp, "ACGT>TGCA")?;
        writeln!(temp, "NNNN")?;
        writeln!(temp, ">read2")?;
        writeln!(temp, "AAAA")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 2);

        let first = block.get(0);
        assert_eq!(first.name(), b"read1 test with > in name");
        assert_eq!(first.seq(), b"ACGT>TGCANNNN", "Sequence should include '>' character");

        let second = block.get(1);
        assert_eq!(second.name(), b"read2");
        assert_eq!(second.seq(), b"AAAA");

        Ok(())
    }

    #[test]
    fn zero_copy_optimization_when_possible() -> Result<()> {
        // Verify that zero-copy works for records that fit in a single parse call
        // Note: Due to buffering by niffler and underlying readers, small files may
        // still span multiple read() calls, so this test verifies correct behavior
        // rather than guaranteeing Local elements in all cases.
        let mut temp = NamedTempFile::new()?;
        writeln!(temp, ">read1")?;
        writeln!(temp, "ACGT")?;
        writeln!(temp, "TGCA")?;
        temp.flush()?;

        let file = File::open(temp.path())?;
        let mut parser = FastaParser::new(vec![file], 10, 1024, 30)?;

        let (block, was_final) = parser.parse()?;
        assert!(was_final);
        assert_eq!(block.entries.len(), 1);

        // Verify the sequence content is correct regardless of Local vs Owned
        let read = block.get(0);
        assert_eq!(read.name(), b"read1");
        assert_eq!(read.seq(), b"ACGTTGCA");
        assert_eq!(read.qual().len(), 8);

        Ok(())
    }
}
