---
weight: 15
title: "Parser Architecture"
---

# Parser Architecture

## Overview

mbf-fastq-processor uses a custom-built parser designed for high performance and correctness when processing FASTQ, FASTA, and BAM files. The parser's design emphasizes:

1. **Zero-copy parsing** where possible to minimize memory allocations
2. **Streaming architecture** to handle files of any size
3. **Transparent compression** support (raw, gzip, zstd)
4. **Stateful parsing** to handle reads spanning block boundaries
5. **Cross-platform compatibility** (Unix/Windows line endings)

## The Zero-Copy Challenge with Compressed Files

### Why Not Pure Zero-Copy?

A common optimization in bioinformatics tools is "zero-copy" parsing, where the parser operates directly on memory-mapped file contents without allocating separate buffers. This works well for uncompressed files stored on fast storage.

However, **FASTQ files are almost always compressed in practice**. This creates a fundamental limitation:

- **Compressed files require decompression buffers**: You cannot memory-map a gzip or zstd file and directly access uncompressed content
- **Decompression is inherently a copy operation**: Data must be decompressed from the file into a buffer
- **Block-based decompression**: Compression algorithms work on blocks, requiring buffer management

Given this reality, mbf-fastq-processor takes a pragmatic approach: **use zero-copy techniques where beneficial after decompression**, rather than forcing a pure zero-copy design that doesn't match real-world usage.

## Hybrid Zero-Copy Architecture

### The FastQElement Design

The parser uses a hybrid memory model implemented through the `FastQElement` enum (defined in `src/io/reads.rs`):

```rust
pub enum FastQElement {
    Owned(Vec<u8>),      // Used for partial/modified reads
    Local(Position),     // Zero-copy references into shared buffer
}

pub struct Position {
    pub start: usize,
    pub end: usize,
}
```

**How it works:**

1. **Most reads are `Local`**: They store only start/end positions referencing a large shared buffer
2. **Some reads are `Owned`**: When a read spans block boundaries or is modified, it gets its own allocation
3. **The shared buffer acts as an arena**: One large `Vec<u8>` holds data for hundreds or thousands of reads

**Memory efficiency:**

- A typical `Position` is 16 bytes (two `usize` values)
- An `Owned` allocation requires heap memory proportional to read length
- For a block of 1000 reads, this can reduce memory by 10x or more compared to allocating each read separately

### Block-Based Processing

The parser operates on blocks of data (`src/io/reads.rs`):

```rust
pub struct FastQBlock {
    pub block: Vec<u8>,          // Shared arena for all reads
    pub entries: Vec<FastQRead>,  // Metadata about each read
}

pub struct FastQRead {
    pub name: FastQElement,
    pub seq: FastQElement,
    pub qual: FastQElement,
}
```

**Processing flow:**

1. Read a large block (typically 128 KB) from the decompressed stream
2. Parse all complete reads in the block, storing positions as `Local` references
3. If a read is incomplete at block boundary, mark it and continue to next block
4. Partial reads become `Owned` when completed across blocks

This design achieves near-zero-copy performance for the common case (complete reads within blocks) while gracefully handling edge cases (reads spanning blocks).

## File Format and Compression Handling

### Automatic Format Detection

mbf-fastq-processor automatically detects file formats by examining magic bytes (`src/io/input.rs`):

```rust
pub enum InputFile {
    Fastq(File),  // Detected by '@' prefix
    Fasta(File),  // Detected by '>' prefix
    Bam(File),    // Detected by 'BAM\x01' magic bytes
}
```

This happens transparently—users never specify format explicitly.

### Transparent Decompression

Compression is handled by the `niffler` crate, which:

1. Examines file headers to detect compression type
2. Wraps the file in an appropriate decompressor
3. Returns a `Box<dyn Read>` that transparently decompresses

**Supported compressions:**
- **Raw** (uncompressed)
- **Gzip** (`.gz`, `.gzip`)
- **Zstandard** (`.zst`, `.zstd`)

**Implementation** (`src/io/input.rs`):

```rust
let (mut reader, _format) = niffler::send::get_reader(Box::new(file))?;
// reader now transparently decompresses on read operations
```

The parser simply reads from this stream—it's completely unaware of compression. This separation of concerns keeps the parser simple while supporting multiple formats.

### Why Compression Matters for Parser Design

The prevalence of compressed FASTQ files shaped our architectural choices:

1. **Buffer allocation is unavoidable**: Decompression requires buffers anyway, so we optimize buffer reuse rather than avoiding buffers entirely
2. **Block boundaries are natural**: Compression works on blocks, making block-based parsing a good fit
3. **Streaming is essential**: Memory-mapping doesn't work with compression; streaming is the natural model
4. **Performance comes from reducing allocations**: The hybrid `Local`/`Owned` approach minimizes heap allocations while accepting that some buffering is necessary

## Stateful Parsing for Read Boundaries

### The Partial Read Problem

When parsing in blocks, reads can be split across boundaries:

```
Block 1: @read1\nACGT\n+\nIIII\n@read2\nAC
Block 2: GT\n+\nII
```

Here `read2` spans both blocks. The parser must:

1. Detect incomplete reads at block end
2. Store partial state
3. Continue parsing when next block arrives

### State Machine Implementation

The parser uses an explicit state machine (`src/io/parsers/fastq.rs`):

```rust
pub enum PartialStatus {
    NoPartial,
    InName,
    InSeq,
    InSpacer,
    InQual,
    // Windows mode variants for CRLF line endings
    InNameNewline,
    InSeqNewline,
    InSpacerNewline,
    InQualNewline,
}
```

**State tracking:**

- After parsing each block, record what state the parser is in
- If incomplete, the partial read becomes `Owned` (stored in `last_read1`, `last_read2`)
- Next block continues from that state, appending data to the owned buffer
- When read completes, it's added to results

This ensures correctness even with arbitrarily positioned block boundaries.

## Cross-Platform Line Ending Support

### Unix vs Windows

FASTQ files may use:
- **Unix**: Line feed (`\n`, 1 byte)
- **Windows**: Carriage return + line feed (`\r\n`, 2 bytes)

### Automatic Detection

The parser automatically detects line endings on the first block (`src/io/parsers/fastq.rs`):

```rust
let windows_mode = match windows_mode {
    Some(x) => x,
    None => {
        // First block: detect by searching for \r
        memchr::memchr(b'\r', &input[pos..stop]).is_some()
    }
};
```

Once detected, it uses the appropriate newline finder:
- Unix: searches for `\n` (1-byte newlines)
- Windows: searches for `\r\n` (2-byte newlines)

The state machine has separate states for Windows mode to correctly handle partial reads at CRLF boundaries.

## Performance Optimizations

### Vectorized Newline Search

The parser uses the `memchr` crate for fast newline detection:

```rust
memchr::memmem::find_iter(&input[pos..stop], b"\n")
```

This library uses SIMD instructions when available, making newline scanning much faster than byte-by-byte iteration.

### Buffer Reuse

Blocks are reused across iterations:
1. Parse block, extract reads
2. When done with block, reuse the `Vec<u8>` buffer for next block
3. Only reallocate if block size changes

This amortizes allocation costs across the entire file.

### Minimal Allocations for Common Case

For reads that don't span blocks (>99% in practice):
- No allocation for sequence data (just positions)
- One allocation per block for the shared buffer
- One allocation per read for the `FastQRead` metadata struct

For reads spanning blocks:
- Additional allocations proportional to number of boundary-spanning reads
- In practice, very rare (a few per megabyte of data)

## Multiple Format Support

### FASTA Parsing

FASTA format is handled by the `bio` crate's parser (`src/io/parsers/fasta.rs`):

- Reads sequences using `bio::io::fasta::Reader`
- Generates synthetic quality scores (configured via `fasta_fake_quality`)
- Converts to FASTQ-compatible representation for pipeline processing

### BAM Parsing

BAM format uses the `noodles` crate (`src/io/parsers/bam.rs`):

- Parses aligned/unaligned reads using `noodles::bam`
- Filters based on `bam_include_mapped` / `bam_include_unmapped` settings
- Extracts sequences and quality scores
- Converts to FASTQ format for uniform pipeline processing

### Parser Trait

All parsers implement a common trait:

```rust
pub trait Parser: Send {
    fn read_block(&mut self, ...) -> Result<()>;
    // ... other methods
}
```

This allows the main processing loop to work with any input format uniformly.

## Summary

The parser architecture reflects real-world constraints:

1. **Compression is ubiquitous**: Design assumes decompression buffers exist
2. **Zero-copy where it matters**: Use references within decompressed blocks
3. **Graceful degradation**: Fall back to owned allocations for edge cases
4. **Stateful parsing**: Handle block boundaries correctly
5. **Format flexibility**: Support multiple input formats transparently

This hybrid approach delivers excellent performance (minimal allocations) while maintaining correctness (proper handling of all edge cases) and flexibility (multiple formats, transparent compression).

The result is a parser that's both fast and correct—suitable for production use on real-world FASTQ files of any size.
