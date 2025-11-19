# Rapidgzip Technique Applied to Zstd: Feasibility Analysis

## Executive Summary

**Can we apply rapidgzip's technique to zstd?** **No, not for arbitrary zstd files.** The fundamental difference in how deflate (gzip) and zstd handle block dependencies makes the rapidgzip approach infeasible for standard zstd compressed files.

## Rapidgzip's Core Innovation

### How Rapidgzip Works

Rapidgzip achieves parallel decompression of **arbitrary** gzip files through:

1. **Trial-and-Error Block Discovery**
   - Speculatively attempts decompression at candidate positions throughout the file
   - Validates attempts via CRC32 checksums and stream integrity checks
   - Builds a map of valid deflate block boundaries

2. **Speculative Parallel Decompression**
   - Multiple threads decompress different blocks simultaneously
   - Uses cache and prefetcher architecture to handle faulty decompression results
   - Reassembles output in correct order

3. **Self-Stabilizing Architecture**
   - Can tolerate decompression failures from invalid starting positions
   - Falls back to sequential decompression when needed
   - Performance: 8.7 GB/s with 128 cores (55× speedup over single-threaded gzip)

**Key enabler**: Deflate blocks can be decompressed with an "empty" or "cold" initial state, allowing resynchronization from arbitrary block boundaries.

**Source**: [Rapidgzip Paper - HPDC '23](https://arxiv.org/abs/2308.08955)

## Why This Doesn't Work for Zstd

### Block Dependency Analysis

#### Deflate (Gzip) Dependencies
```
Block N:
├── LZ77 Window: References previous 32KB of decoded data
├── Huffman Trees: Can be reset per block
└── Can start with empty window → limited quality loss
```

**Critical property**: A deflate decoder can begin at a block boundary with an empty window and still produce valid (though potentially suboptimal) output.

#### Zstd Dependencies
```
Block N:
├── LZ77 Window: References up to Window_Size (128KB - 128MB) of previous decoded data
├── Recent Offsets: Last 3 offset values from previous blocks
├── Huffman Trees: May use previous block's tree (treeless mode)
├── FSE Tables: May reuse previous block's tables (repeat mode)
└── Cannot start without prior state → decompression fails
```

**Critical difference**: Zstd format specification states:

> "Each block depends on previous blocks for proper decoding. However, each block can be decompressed without waiting for its successor, allowing streaming operations."

From the [zstd format specification](https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md), to decode any block you need:
- Previous decoded data (up to Window_Size distance)
- Recent offsets from prior compressed blocks
- Previous Huffman tree (for treeless literal blocks)
- Prior FSE decoding tables (when repeat modes are used)

**Bottom line**: You cannot start decompressing a zstd block from an arbitrary position without the full decompression state from all previous blocks.

### Attempted Workarounds and Why They Fail

#### Option 1: "Try anyway and see what works"
**Problem**: Unlike deflate where an empty window produces degraded but valid output, zstd decompression will:
- Fail to decode FSE/Huffman tables in repeat mode
- Produce incorrect LZ77 backreferences
- Generate corrupt output that cannot be validated

#### Option 2: "Find frame boundaries instead of block boundaries"
**Partial solution**: Zstd frames ARE independent, but:
- **For standard zstd files**: Typically one large frame per file
- **For pzstd files**: Multiple frames, but requires special compression
- **For arbitrary files**: Most contain single frames, providing no parallelism opportunity

This is why pzstd only works on files it compressed itself.

#### Option 3: "Build an index while compressing"
**Status**: Already exists! This is the **seekable zstd format**:
- `indexed_zstd` (Python) by Marco Martinelli
- `zeekstd` (Rust)
- `zstd-seekable-format-go` (Go)

**Limitation**: Requires preprocessing - cannot parallelize arbitrary zstd files.

## Comparison Table

| Feature | Rapidgzip (Deflate) | Zstd Equivalent |
|---------|---------------------|-----------------|
| Works on arbitrary files | ✅ Yes | ❌ No |
| Requires preprocessing | ❌ No | ✅ Yes (seekable format) |
| Block independence | ⚠️ Partial (can start with empty window) | ❌ No (requires full prior state) |
| Parallel decompression | ✅ Yes (55× speedup) | ⚠️ Only with special formats |
| Trial-and-error viable | ✅ Yes (validation via CRC) | ❌ No (produces corrupt output) |

## What IS Possible with Zstd

### 1. Seekable Format (Preprocessing Required)

**Available implementations**:
- **indexed_zstd**: Python library using libzstd-seek
- **zeekstd**: Rust implementation of seekable format
- **t2sz**: Compression tool creating seekable archives

**How it works**:
```
File compressed with seekable format:
[Frame 1][Skippable Frame Header][Frame 2][Skippable Frame Header]...[Seek Table]
                                                                      └─ Magic: 0x8F92EAB1
```

Each frame is independent and can be decompressed in parallel.

**Workflow**:
```bash
# Compress with seekable format
t2sz input.fastq -o output.fastq.zst

# Decompress frames in parallel (manual implementation needed)
# Thread 1: Frame 0
# Thread 2: Frame 1
# Thread 3: Frame 2
# ...
```

**Rust crate**: `zeekstd = "0.1.0"`

**Performance**: Potentially high, but:
- Requires all files to be recompressed
- Slightly worse compression ratio (lost inter-frame correlations)
- Not applicable to existing zstd files

### 2. Multi-File Parallelization (Already Applicable)

**For your use case** (mbf-fastq-processor):

Current code in `src/io/parsers/fastq.rs:48-50`:
```rust
if let Some(next_file) = self.readers.pop() {
    let (reader, _format) = niffler::send::get_reader(Box::new(next_file))?;
    self.current_reader = Some(reader);
}
```

**Opportunity**: When processing multiple input files:
```toml
[input]
read1 = ['sample1.fastq.zst', 'sample2.fastq.zst', 'sample3.fastq.zst']
```

Could decompress each file in parallel on different threads, then combine streams.

**Expected speedup**: ~N× for N files (limited by available cores)

### 3. Alternative Format: LZ4

**Trade-off consideration**:
- **Decompression speed**: 2-3× faster than zstd
- **Compression ratio**: ~15-20% worse than zstd
- **Already supported**: Via niffler

**Benchmark data** (from search results):
```
Format      | Compression | Decompression | Ratio
------------|-------------|---------------|-------
gzip        | 8.5 MB/s    | 250 MB/s      | 3.3×
zstd        | 350 MB/s    | 800 MB/s      | 3.5×
lz4         | 650 MB/s    | 2100 MB/s     | 2.8×
```

For I/O-bound workloads where storage is cheap, LZ4 may be optimal.

## Technical Deep Dive: Why Deflate Allows Resynchronization

### Deflate Block Structure
```
Deflate Block:
┌─────────────────────┐
│ Block Header (3 bits)│
│ - BFINAL: last block │
│ - BTYPE: block type  │
├─────────────────────┤
│ Huffman Trees       │ ← Can be rebuilt from block data
│ (for dynamic blocks)│
├─────────────────────┤
│ Compressed Data     │
│ - Literals          │
│ - Length/Distance   │
│   pairs (LZ77)      │
└─────────────────────┘
```

**Key properties**:
1. Each block contains its own Huffman tree (for dynamic blocks) or uses standard trees (for fixed blocks)
2. LZ77 backreferences are limited to 32KB window
3. Starting with empty window: backreferences beyond window boundary are simply not used
4. Output is valid (though compression ratio was lower than if decompressed sequentially)

### Zstd Block Structure
```
Zstd Block:
┌────────────────────────┐
│ Block Header (3 bytes) │
│ - Block_Type           │
│ - Block_Size           │
├────────────────────────┤
│ Literals Section       │
│ - May use "Repeat"     │ ← Requires previous FSE table!
│   FSE table mode       │
│ - May use "Treeless"   │ ← Requires previous Huffman tree!
│   Huffman mode         │
├────────────────────────┤
│ Sequences Section      │
│ - May use "Repeat"     │ ← Requires previous FSE table!
│   mode for tables      │
│ - Offset references    │ ← Requires last 3 offset values!
│ - LZ77 backrefs        │ ← Requires full window (up to 128MB)!
└────────────────────────┘
```

**Key blocking issues**:
1. **Repeat modes**: Block may not contain its own FSE/Huffman tables
2. **Large windows**: Up to 128MB of previous data needed for backreferences
3. **Offset history**: Requires previous 3 offset values for sequence decoding
4. **No "cold start" mode**: Format has no provision for starting without prior state

## Author's Perspective

The rapidgzip author (mxmlnkn) addressed zstd in the **ratarmount** project:

> "Zstd support is provided by indexed_zstd by Marco Martinelli"

This indicates even the rapidgzip author uses the **seekable format approach** for zstd rather than attempting to apply the rapidgzip technique.

Performance comparison from ratarmount benchmarks:
```
Format          | Indexed Access | Parallel Decompression
----------------|----------------|----------------------
gzip (rapidgzip)| ✅ Yes         | ✅ Yes (arbitrary files)
zstd (indexed)  | ✅ Yes         | ⚠️  Only seekable format
```

## Conclusion and Recommendations

### For mbf-fastq-processor

**Short term** (Immediate benefit, no format changes):
1. **Parallelize multi-file decompression**
   - When users provide multiple input files, decompress in parallel
   - Leverage existing crossbeam infrastructure
   - Expected: N× speedup for N files

2. **Document format tradeoffs**
   - Add benchmark data for gzip vs zstd vs lz4
   - Help users choose optimal format for their workflow

**Long term** (Advanced users, preprocessing required):
3. **Add seekable zstd support**
   - Integrate `zeekstd` crate
   - Detect seekable format automatically
   - Decompress frames in parallel using rayon
   - Provide `mbf-fastq-recompress` utility to convert standard → seekable

4. **Benchmark to validate**
   - Measure if decompression is actually the bottleneck
   - Profile with different compression formats
   - Test scalability with multiple cores

### What We Learned

The rapidgzip technique is **format-specific** to deflate's properties:
- ✅ Works for deflate: Blocks can start with empty window
- ❌ Doesn't work for zstd: Blocks require full prior state
- ✅ Alternative exists: Seekable format (preprocessing required)

For **arbitrary** zstd files (the vast majority of real-world files), parallel decompression remains fundamentally constrained by the format's sequential dependencies.

## References

1. [Rapidgzip Paper - HPDC '23](https://arxiv.org/abs/2308.08955) - Knespel & Brunst, 2023
2. [Zstd Format Specification](https://github.com/facebook/zstd/blob/dev/doc/zstd_compression_format.md)
3. [Zstd Seekable Format Spec](https://github.com/facebook/zstd/blob/dev/contrib/seekable_format/zstd_seekable_compression_format.md)
4. [indexed_zstd](https://github.com/martinellimarco/indexed_zstd) - Marco Martinelli
5. [zeekstd](https://github.com/rorosen/zeekstd) - Rust seekable format implementation
6. [Multithreaded Decompression Issue #2470](https://github.com/facebook/zstd/issues/2470)
