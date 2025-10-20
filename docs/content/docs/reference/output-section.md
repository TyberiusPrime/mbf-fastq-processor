---
weight: 4
not-a-transformation: true
---

# Output section

The `[output]` table controls how transformed reads and reporting artefacts are written.

```toml
[output]
    prefix = "output"          # required.
    format = "Gzip"             # Raw | Gzip | Zstd | Bam | None (default: Raw)
    suffix = ".fq.gz"           # optional override; inferred from format when omitted
    compression_level = 6        # gzip: 0-9, zstd: 1-22, bam: 0-9 (BGZF); defaults are gzip=6, zstd=5
    ix_separator = "_"          # optional separator between prefix, infixes, and segments. Defaults to '_'

    report_json = false          # write prefix.json
    report_html = true           # write prefix.html

    output = ["read1", "read2"] # limit which segments become FastQ files
    interleave = false           # emit a single interleaved FastQ
    stdout = false               # stream to stdout instead of files

    output_hash_uncompressed = false
    output_hash_compressed = false
```

| Key                     | Default | Description |
|-------------------------|---------|-------------|
| `prefix`                | `"output"` | Base name for all files produced by the run. |
| `format`                | `"Raw"` | Compression applied to read outputs. `Bam` writes an unaligned BAM file, while `None` suppresses FastQ writing but still allows reports. |
| `suffix`                | derived from format | Override file extension when interop with other tooling demands a specific suffix. |
| `compression_level`     | gzip: 6, zstd: 5 | Fine-tune compression effort. Ignored for `Raw`/`None`. `Bam` maps directly to the BGZF level (0–9). |
| `report_json` / `report_html` | `false` | Toggle structured or interactive reports. |
| `output`                | all input segments | Restrict the subset of segments written to disk. Use an empty list to suppress FastQs while still running steps that depend on fragment data. |
| `interleave`            | `false` | Generate a single interleaved FastQ (`{prefix}_interleaved.fq*`).|
| `stdout`                | `false` | Write to stdout. Forces `format = "Raw"`. `Sets interleave=true` if more than one fragment is listed in `output`|
| `output_hash_uncompressed` / `output_hash_compressed` | `false` | Emit SHA-256 checksums. |
| `ix_separator`          | `"_"` | Separator inserted between `prefix`, any infix (demultiplex labels, inspect names, etc.), and segment names. |

Generated filenames join these components with `ix_separator` (default `_`), e.g. `{prefix}_{segment}{suffix}`. Interleaving replaces `segment` with `interleaved`; demultiplexing adds per-barcode infixes before the segment. Checksums use `.uncompressed.sha256` or `.compressed.sha256` suffixes.

Compression format and suffix are independent: overriding the suffix will not change the actual compression algorithm. 

> **BAM-specific notes**
> - `format = "Bam"` emits an *unaligned* BAM file using BGZF compression.
> - BAM may not contain spaces in read names. If a read has a space in it's Fastq name, it's truncated at the first space, and the remaining text is placed in the "CO" tag.
> - BAM output cannot be streamed to stdout and requires `output_hash_uncompressed = false` (compressed hashes continue to work).
> - Interleaved writes produce one paired BAM with appropriate SAM flags; per-segment outputs yield independent BAMs.

### Example output files.

#### As above
The above configuration produces:
- `output_read1.fq.gz` # .fq is the default suffix for raw, .fq.gz for gzip
- `output_read2.fq.gz`
- `output.html` # HTML report

#### If Interleaved was set
- `output_interleaved.fq.gz` 
- `output.html` # HTML report

### No FastQ output

Set `format = "None"` or `output = []`  when you only need reports or tag quantification. A `prefix` is still required so report files have a stable name.

See also the [Report steps reference]({{< relref "docs/reference/report-steps/_index.md" >}}) for producing summaries, and the [Demultiplex documentation]({{< relref "docs/reference/Demultiplex.md" >}}) for how barcode outputs influence file naming.


## Named pipe outputs
Output files may be (preexisting) named pipes (FIFOs).
