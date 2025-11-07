
### Inspect

Dump a few reads to a FASTQ file for inspection at this point in the graph.

```toml
[[step]]
    action = "Inspect"
    n  = 1000 # how many molecules 
    infix = "inspect_at_point" # output filename infix
    segment = "read1" # Any of your input segments (use "all" for interleaved output)
    suffix = "compressed" # (optional) custom suffix for filename
    compression = "gzip" # (optional) compression format: raw, gzip, zstd
    compression_level = 1 # (optional) compression level for gzip/zstd/bam (gzip, zstd: 1-22)
                          # defaults: gzip=6, zstd=5
```

Output filename pattern:
- Without custom suffix:
  - When `segment` names a single read: `{prefix}{ix_separator}{infix}{ix_separator}{segment}.{format_extension}`
  - When `segment = "all"`: `{prefix}{ix_separator}{infix}{ix_separator}interleaved.{format_extension}`
- With a custom suffix, replace `{format_extension}` with the explicit suffix (e.g. `{prefix}{ix_separator}{infix}{ix_separator}{segment}.compressed`).

Where `{format_extension}` is:
- `fq` for raw format
- `fq.gz` for gzip format  
- `fq.zst` for zstd format
- `bam` for BAM format


Note that inspect will collect all reads in memory before writing them out.
When `segment = "all"` the collected reads are written in interleaved order
(`read1`, `read2`, … per molecule).
