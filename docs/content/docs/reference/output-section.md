---
weight: 4
not-a-transformation: true
---

# Output section

The `[output]` table controls how transformed reads and reporting artefacts are written.

```toml
[output]
    prefix = "output"          # defaults to "output"
    format = "Gzip"             # Raw | Gzip | Zstd | None (default: Raw)
    suffix = ".fq.gz"           # optional override; inferred from format when omitted
    compression_level = 6        # gzip: 0-9, zstd: 1-22; defaults are gzip=6, zstd=5

    report_json = false          # write prefix.json
    report_html = true           # write prefix.html

    output = ["read1", "read2"] # limit which segments become FastQ files
    interleave = false           # emit a single interleaved FastQ
    stdout = false               # stream read1 to stdout instead of files

    output_hash_uncompressed = false
    output_hash_compressed = false
```

| Key                     | Default | Description |
|-------------------------|---------|-------------|
| `prefix`                | `"output"` | Base name for all files produced by the run. |
| `format`                | `"Raw"` | Compression applied to FastQ outputs. `None` disables FastQ emission but still allows reports. |
| `suffix`                | derived from format | Override file extension when interop with legacy tooling demands a specific suffix. |
| `compression_level`     | gzip: 6, zstd: 5 | Fine-tune compression effort. Ignored for `Raw`/`None`. |
| `report_json` / `report_html` | `false` | Toggle structured or interactive reports. |
| `output`                | all input segments | Restrict the subset of segments written to disk. Use an empty list to suppress FastQs while still running steps that depend on fragment data. |
| `interleave`            | `false` | Generate a single interleaved FastQ (`{prefix}_interleaved.fq*`). Mutually exclusive with `stdout = true`. |
| `stdout`                | `false` | Write the first listed segment to stdout. Forces `format = "Raw"` and enables interleaving for paired reads. |
| `output_hash_uncompressed` / `output_hash_compressed` | `false` | Emit SHA-256 checksums for quality control. |

Generated filenames follow `{prefix}_{segment}{suffix}` unless interleaving or demultiplexing steps pick alternative infixes. Checksums use `.uncompressed.sha256` or `.compressed.sha256` suffixes.

Compression format and suffix are independent: overriding the suffix will not change the actual compression algorithm. Keep them aligned to avoid confusing downstream tools.

### No FastQ output

Set `format = "None"` or `output = []` to run in readless mode—for example, when you only need reports or tag quantification. A `prefix` is still required so report files have a stable home.

See also the [Report steps reference]({{< relref "docs/reference/report-steps/_index.md" >}}) for producing summaries, and the [Demultiplex documentation]({{< relref "docs/reference/Demultiplex.md" >}}) for how barcode outputs influence file naming.
