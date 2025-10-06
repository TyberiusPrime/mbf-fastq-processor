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
| `interleaved` | No | array of strings | Enables interleaved reading; must list segment names in their on-disk order. |

Additional points:

- Segment names are user-defined; adopt consistent casing because downstream `segment` arguments must match.
- Compression is auto-detected for `.gz`, `.bz2`, and `.zst` extensions by inspecting file headers.
- Every segment must provide the same number of reads. Length mismatches raise a validation error before processing begins.
- Multiple files per segment are concatenated virtually; the processor streams them sequentially.

## Interleaved input

Some datasets store all segments in a single file. Activate interleaved mode to describe how the segments are ordered:

```toml
[input]
    source = ['interleaved.fq']
    interleaved = ["read1", "read2", "index1", "index2"]
```

Rules for interleaving:

- The `[input]` table must contain exactly one data source when `interleaved` is present.
- The `interleaved` list dictates how reads are grouped into fragments. The length of the list equals the number of segments.
- Downstream steps reference the declared segment names exactly as written in the list.

Need to ingest barcodes from separate files? Combine interleaved input with additional per-segment files by using a demultiplex step that reads from tags, or keep traditional multi-file input for clarity.
