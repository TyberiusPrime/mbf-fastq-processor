# Parallel Zstd Decompression Research

## Executive Summary

**Bottom Line:** Native parallel zstd decompression for standard zstd files is **not currently available** in any production-ready form. However, there are several alternative approaches worth considering.

## Current Project Setup

The mbf-fastq-processor currently uses:
- **niffler 3.0**: Auto-detection and handling of multiple compression formats (gzip, zstd, etc.)
- **zstd 0.13.2**: Zstd compression/decompression (bindings to C library)
- **crossbeam 0.8.4**: Already used for parallel processing via channels

### Current I/O Architecture
The codebase in `src/io/parsers/fastq.rs:49` uses:
```rust
let (reader, _format) = niffler::send::get_reader(Box::new(next_file))?;
```
This provides automatic format detection but single-threaded decompression.

## State of Parallel Zstd Decompression

### 1. Native Zstd Library Limitations

**Official Position:** The zstd maintainer (Yann Collet) has stated:
- "There is currently no multithreading for decompression, only for compression"
- Rationale was that single-threaded decompression was "already plenty fast with just a single thread (typically faster than ssd)"
- However, this is **outdated** for modern hardware:
  - PCIe 4.0 SSDs: 7000+ MiB/s read
  - PCIe 5.0 SSDs: 12,400+ MiB/s sequential read
  - Single-threaded zstd decompress: typically 500-1500 MiB/s

**Source:** [GitHub Issue #2470](https://github.com/facebook/zstd/issues/2470)

### 2. PZstd (Parallel Zstandard)

**Status:** Contrib tool in zstd repository, now **obsoleted** by `zstd -T0`

**How it works:**
- **Compression:** Splits input into equal-sized chunks, compresses each independently into separate zstd frames
- **Frame Headers:** Writes 12-byte skippable frames between chunks to mark boundaries
- **Parallel Decompression:** Only works for files compressed with pzstd itself

**Limitation:** Standard zstd files cannot be decompressed in parallel by pzstd. For standard files, it only uses separate threads for IO and decompression (2 threads total).

### 3. Seekable Zstd Format

**Available Rust Crates:**

#### zeekstd (v0.1.0)
- **URL:** https://github.com/rorosen/zeekstd
- **Purpose:** Rust implementation of Zstandard Seekable Format
- **How it works:**
  - Splits compressed data into independent frames
  - Maintains a seek table for frame locations
  - Enables random access without decompressing entire file
- **Parallel potential:** Independent frames *could* be decompressed in parallel using rayon, but this is not implemented out-of-the-box
- **Limitation:** Requires files to be compressed in seekable format

#### zstd-seekable (v0.1.4)
- **Function:** Has `parallel_compress` for compression
- **Status:** No parallel decompression documented

#### zstd-framed
- **Purpose:** Seekable reader/writer for zstd streams with multiple frames
- **Features:** Sync and async I/O support
- **Limitation:** No explicit parallel decompression

### 4. Pure Rust Implementations

#### ruzstd (v0.7.4)
- **URL:** https://github.com/KillingSpark/zstd-rs
- **Language:** Pure Rust implementation of RFC8878
- **Status:** Fully operational decompression
- **Parallelism:** No multi-threaded decompression mentioned
- **Advantage:** No C dependencies

## Potential Approaches for mbf-fastq-processor

### Option 1: Keep Current Single-Threaded Approach
**Effort:** None
**Performance gain:** 0%
**Rationale:** May already be I/O bound for slower storage

### Option 2: Decompress Multiple Input Files in Parallel
**Effort:** Low-Medium
**Performance gain:** Significant (if multiple input files present)
**Implementation:**
- Currently, the project reads multiple input files sequentially via `ChainedParser`
- Could spawn parallel decompression threads for each input file
- Combine streams afterward
- **Best for:** Workflows with multiple input files (e.g., `read1 = ['fileA.fastq.zstd', 'fileB.fastq.zstd', 'fileC.fastq.zstd']`)

**Code location to modify:**
- `src/io/parsers/fastq.rs:48-50` - where files are processed sequentially
- `src/pipeline.rs:23-43` - `parse_and_send` function

### Option 3: Use Seekable Format with Manual Parallelization
**Effort:** High
**Performance gain:** Potentially significant for large single files
**Implementation:**
1. Add `zeekstd` as dependency
2. Detect if file is in seekable format
3. Use rayon to decompress independent frames in parallel
4. Recombine in order

**Challenges:**
- Requires files to be compressed in seekable format (preprocessing step)
- More complex implementation
- Ordering overhead

### Option 4: Switch to LZ4 for Speed-Critical Workflows
**Effort:** Low (niffler already supports it)
**Performance gain:** 2-3x decompression speed vs zstd
**Tradeoff:** Larger file sizes (worse compression ratio)
**Recommendation:** Document as option for users prioritizing speed

### Option 5: Experiment with Gzip + Pigz/libdeflate
**Note:** The comment in `dev/others/seqstats.md:8` mentions:
> "Pigz does it in 12.8s, so you *can* parallel decompress gzip.."

**Effort:** Medium
**Implementation:** Use `libdeflate` crate which supports multi-threaded gzip decompression
**Tradeoff:** Gzip typically has worse compression than zstd

## Benchmark Recommendation

Before implementing any solution, benchmark current I/O performance:
1. Measure if decompression is actually the bottleneck
2. Test with different compression formats (raw, gzip, zstd, lz4)
3. Test with single vs. multiple input files
4. Profile using existing `timing` module (`src/timing.rs`)

### Suggested Benchmark Command
```bash
# Add timing output
cargo build --release

# Test different compression formats
time ./target/release/mbf-fastq-processor <config_zstd.toml>
time ./target/release/mbf-fastq-processor <config_gzip.toml>
time ./target/release/mbf-fastq-processor <config_raw.toml>

# Check CPU utilization during zstd decompression
htop  # Look for single-core saturation
```

## Recommendations

### Immediate (Low Effort, High Impact)
1. **Document compression format tradeoffs** in README:
   - zstd: Best compression, single-threaded decompression
   - gzip: Medium compression, limited parallel decompression via pigz
   - lz4: Fastest decompression, larger files
   - raw: No overhead, but large storage

2. **Benchmark current performance** to identify if decompression is actually a bottleneck

### Short Term (If Decompression is Bottleneck)
3. **Implement parallel multi-file decompression** (Option 2):
   - Highest ROI for users processing multiple files
   - Leverages existing crossbeam infrastructure
   - No format changes required

### Long Term (If Large Single Files are Common)
4. **Add seekable zstd support** (Option 3):
   - For advanced users with very large single files
   - Provide utility to convert standard zstd → seekable zstd
   - Requires user opt-in (preprocessing step)

## References

- [Zstd Parallel Decompression Issue #2470](https://github.com/facebook/zstd/issues/2470)
- [Zstd Multi-threading Support Issue #491](https://github.com/facebook/zstd/issues/491)
- [Zeekstd - Rust Seekable Format](https://github.com/rorosen/zeekstd)
- [Ruzstd - Pure Rust Implementation](https://github.com/KillingSpark/zstd-rs)
- [Niffler - Format Detection Library](https://github.com/luizirber/niffler)

## Rapidgzip-Style Approach for Zstd?

After investigating the rapidgzip paper and technique (see `rapidgzip_vs_zstd_analysis.md`), the conclusion is:

**The rapidgzip approach CANNOT be applied to arbitrary zstd files** due to fundamental format differences:

- **Rapidgzip works** because deflate blocks can be decompressed starting with an "empty" window
- **Zstd blocks require** full prior decompression state (FSE tables, Huffman trees, offset history, full LZ77 window)
- **Trial-and-error fails** because zstd cannot "resynchronize" mid-stream like deflate can

Even the rapidgzip author (mxmlnkn) uses indexed_zstd with the seekable format for zstd support in ratarmount, rather than attempting to apply the rapidgzip technique.

## Conclusion

**No drop-in parallel zstd decompression solution exists** for standard zstd files. The most practical approaches are:
1. Decompress multiple input files in parallel (requires code changes, benefits existing workflows)
2. Switch to LZ4 for speed-critical cases (no code changes, user decision)
3. Use seekable zstd format (requires preprocessing, complex implementation)

The best path forward depends on:
- Whether decompression is actually the bottleneck (needs benchmarking)
- Whether users typically process single large files or multiple smaller files
- Whether the compression ratio vs. speed tradeoff matters for storage/transfer
