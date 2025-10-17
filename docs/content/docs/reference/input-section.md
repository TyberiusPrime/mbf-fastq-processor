---
weight: 3
not-a-transformation: true
---

# Input section

The `[input]` table enumerates all FastQ sources that make up a fragment. At least one segment must be declared.

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

- Segment names are user-defined and case sensitive. Common conventions include `read1`, `read2`, `index1`, and `index2`.
- Compression is auto-detected for by inspecting file headers.
- Every segment must provide the same number of reads. Cardinality mismatches raise a validation error.
- Multiple files per segment are concatenated virtually; the processor streams them sequentially.
- The segment name 'All' is reserved, since some steps use it to signal working on all segments.
- The segment name 'options' (any casing) is reserved for `[input.options]` and cannot be used as a segment key.

## Input options

Format-specific behaviour is configured via the optional `[input.options]` table. These knobs are required when the corresponding file types are present and ignored otherwise.

```toml
[input]
    read1 = ["reads.fasta"]

[input.options]
    fasta_fake_quality = 'a'        # required for FASTA inputs: synthetic Phred score to apply to every base. Used verbatim without further shifting.
    bam_include_mapped = true      # required for BAM inputs: include reads with a reference assignment
    bam_include_unmapped = true    # required for BAM inputs: include reads without a reference assignment
```

- `fasta_fake_quality` accepts a byte character or a number and is used verbatim. Stick to Phred ('!'/33 = worst).
  The value must be supplied whenever any FASTA source is detected.
- `bam_include_mapped` and `bam_include_unmapped` must both be defined when reading BAM files. At least one of them has to be `true`; disabling both would discard every record.
- Format detection is automatic and based on magic bytes: BAM (`BAM\x01`), FASTA (`>`), and FASTQ (`@`).

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

By default, if multiple segmens are defined, every 1000th read pair is checked for the read name prefix (up until the first /)
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
Input files may be named pipes (FIFOs) - but only FastQ formated data is supported in that case.
