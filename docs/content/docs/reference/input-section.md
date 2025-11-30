---
weight: 3
not-a-transformation: true
---

# Input section

The `[input]` table enumerates all read sources that make up a fragment. 
At least one segment must be declared.

```toml
[input]
    read1 = ['fileA_1.fastq', 'fileB_1.fastq.gz', 'fileC_1.fastq.zst'] # required: one or more paths
    read2 = "fileA_2.fastq.gz"                                      # optional
    index1 = ['index1_A.fastq.gz']                                   # optional
    # interleaved = [...]                                            # optional, see below
```

| Key         | Required | Value type       | Notes |
|-------------|----------|------------------|-------|
| segment name (e.g. `read1`) | Yes (at least one) | string or array of strings | Each unique key defines a segment; arrays concatenate multiple files in order. |
| `interleaved` | No | array of strings | Enables interleaved reading; must list segment names in their in-file order. |

Additional points:

- Segment names are user-defined and case sensitive. Common conventions include `read1`, `read2`, `index1`, and `index2`. They must conform to `[a-zA-Z0-9_]+$`.
- Compression is auto-detected for by inspecting file headers.
- Supported file formats are FASTQ, FASTA, and BAM. See [Input options](#input-options) below for format-specific settings.
- Every segment must provide the same number of reads. Cardinality mismatches raise a validation error.
- Multiple files per segment are concatenated virtually; the processor streams them sequentially.
- The names 'All' and 'options' can not be used for segment names.

## File Formats

mbf-fastq-processor supports multiple input formats with automatic detection and transparent decompression.

### Supported Formats

| Format | Detection Method | Compression Support | Notes |
|--------|------------------|---------------------|-------|
| **FASTQ** | First byte is `@` | Raw, Gzip, Zstd | Primary format, fully optimized parser |
| **FASTA** | First byte is `>` | Raw, Gzip, Zstd | Converted to FASTQ with synthetic quality scores |
| **BAM** | Magic bytes `BAM\x01` | Built-in (BAM format) | Aligned and unaligned reads supported |

### Compression Formats

Compression is automatically detected by examining file headers—no need to specify format explicitly:

- **Raw** (uncompressed): `.fastq`, `.fq`, `.fasta`, `.fa`
- **Gzip**: `.gz`, `.gzip` (most common)
- **Zstandard**: `.zst`, `.zstd` (faster compression/decompression)

**Example filenames that work automatically:**
- `reads.fastq`, `reads.fastq.gz`, `reads.fq.zst`
- `input.fasta`, `genome.fa.gz`
- `aligned.bam`, `unaligned.bam`

### FASTQ Format Requirements

FASTQ files should follow the standard format described by [Cock et al. (2010)](https://academic.oup.com/nar/article/38/6/1767/3112533):

```
@read_name optional_comment
ACGTACGTACGT
+
IIIIIIIIIIII
```

- **Line 1**: `@` followed by read identifier, optionally with comments after a separator (default: space)
- **Line 2**: DNA/RNA sequence (A, C, G, T, N, and IUPAC ambiguity codes)
- **Line 3**: `+` optionally followed by repeat of identifier (content ignored)
- **Line 4**: Quality scores (Phred+33 encoding standard)

**Line endings**: Both Unix (`\n`) and Windows (`\r\n`) line endings are automatically detected and handled correctly.

### FASTA Format

FASTA files are converted to FASTQ format for processing:

- Sequences are read normally
- Quality scores are synthesized using the `fasta_fake_quality` setting
- All downstream processing treats them as FASTQ

Required configuration when using FASTA:

```toml
[input.options]
    fasta_fake_quality = 'I'  # or numeric value (33-126)
```

The quality character should be chosen based on your quality filtering requirements. Common values:
- `'I'` (73): High quality (Q40)
- `'?'` (63): Medium quality (Q30)
- `'!'` (33): Minimum quality (Q0)

### BAM Format

BAM files (Binary Alignment Map) are supported with flexible filtering:

```toml
[input.options]
    bam_include_mapped = true      # Include aligned reads
    bam_include_unmapped = true    # Include unaligned reads
```

Both settings must be specified when using BAM input. At least one must be `true`.

**Use cases:**
- Extract unmapped reads from aligned BAM files for reanalysis
- Process all reads (mapped + unmapped) together
- Filter only aligned reads for downstream analysis

Quality scores are extracted directly from BAM records. Sequences are output in their stored orientation (may be reverse-complemented if aligned to reverse strand).

### Parser Architecture

For technical details about how parsing works, including the zero-copy design and handling of compressed files, see [Parser Architecture]({{< relref "docs/concepts/parser-architecture.md" >}}).

**Key implementation features:**
- **Hybrid zero-copy parsing**: Minimizes memory allocations while handling compressed files efficiently
- **Streaming architecture**: Handles files of any size without loading entire file into memory
- **Block-based processing**: Efficient handling of both compressed and uncompressed formats
- **Stateful parsing**: Correctly handles reads spanning block boundaries
    
## Input options

Format-specific behaviour is configured via the optional `[input.options]` table. These knobs are required when the corresponding file types are present and ignored otherwise.

```toml
[input]
    read1 = ["reads.fasta"]

[input.options]
    use_rapidgzip = true          # boolean, defaults to 'automatic'
    build_rapidgzip_index = false # boolean
    fasta_fake_quality = 'a'      # required for FASTA inputs: synthetic Phred score to apply to every base. Used verbatim without further shifting.
    bam_include_mapped = true     # required for BAM inputs: include reads with a reference assignment
    bam_include_unmapped = true   # required for BAM inputs: include reads without a reference assignment
	read_comment_char = ' '       # defaults to ' '. The character seperating read name from the 'read comment'.
```

- `use use_rapidgzip` - whether to decompress gzip with [rapidgzip](https://github.com/mxmlnkn/rapidgzip). 
  See the [rapidgzip section](#rapidgzip).
- `build_rapidgzip_index` - whether to put a rapidgzip index next to your input file if it doesn't exist.
  See the [rapidgzip section](#rapidgzip).

- `fasta_fake_quality` accepts a byte character or a number and is used verbatim. Stick to Phred ('!'/33 = worst).
  The value must be supplied whenever any FASTA source is detected.
- `bam_include_mapped` and `bam_include_unmapped` must both be defined when reading BAM files. At least one of them has to be `true`; disabling both would discard every record.
- Format detection is automatic and based on magic bytes: BAM (`BAM\x01`), FASTA (`>`), and FASTQ (`@`).
- The read_comment_char is used for input reads  
    (e.g. when [`TagDeduplicate`]({{< relref "docs/reference/tag-steps/tag/TagDuplicates.md" >}}) with a name: source). 
    The output steps ([`StoreTagInComment`]({{< relref "docs/reference/tag-steps/using/StoreTagInComment.md" >}}), [`StoreTagLocationInComment`]({{< relref "docs/reference/tag-steps/using/StoreTagLocationInComment.md" >}})) default to this setting, but allow overwriting.

## Interleaved input

Some datasets store all segments in a single file. Activate interleaved mode and describe how the segments are ordered:

```toml
[input]
    source = ['interleaved.fq'] # this 'virtual' segment will not be available for steps downstream
    interleaved = ["read1", "read2", "index1", "index2"]
```

Rules for interleaving:

- The `[input]` table must contain exactly **one** data source when `interleaved` is present.
- The `interleaved` list dictates how reads are grouped into fragments. The length of the list equals the number of segments.
- Downstream steps reference the declared segment names exactly as written in the list.


## Automatic segment (pair) name checking.

By default, if multiple segments are defined, every 1000th read pair is checked for the read name prefix (up until the first /)
matching, ensuring correctly paired reads. 

This assumes Illumina style named reads ending e.g. '/1' and '/2'.

The automatism can be disabled with 

```toml # ignore_in_test
[options]
    spot_check_read_pairing = false
```

To influence the character that delimits the read name prefix, or the sampling rate,
add an explicit [`SpotCheckReadPairing`]({{< relref "docs/reference/validation-steps/SpotCheckReadPairing.md" >}}) step.


## Named pipe input
Input files may be named pipes (FIFOs) - but only FASTQ formated data is supported in that case.


## Rapidgzip

mbf-fastq-processor can use [rapidgzip](https://github.com/mxmlnkn/rapidgzip), a gzip decompression
program that enables multi-core decompression of arbitrary gzip files instead of it's build-in gzip
decompressor.

Since gzip decompression is the single largest bottleneck in FASTQ processing,
this offers massive speed advantages.

By default, we use rapidgzip if a rapidgzip binary is detected on the $PATH and there are 
at least three threads available per segment for decompression (benchmarking indicates rapidgzip
is slower than our build-in gzip decompression otherwise).

You can force rapidgzip use by setting `options.use_rapidgzip` to true, in that case a missing
rapidgzip binary will lead to an error. Likewise, you can disable rapidgzip use by setting it to false.

Rapidgzip can be even faster when there's an index next to the gzip file telling it where
the block starts. We auto-detect and use such an index if it's named `$input_file.rapidgzip_index`.

If `options.build_rapidgzip_index` is set, the index is created if it doesn't exist.
It's placed next to the file. If you expect to run mbf-fastq-processor multiple times on the same
input (such as in development) you might want to spent the disk space.






