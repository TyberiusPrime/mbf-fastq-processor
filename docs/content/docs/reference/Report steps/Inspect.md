
### Inspect

Dump a few reads to a file for inspection at this point in the graph.

```toml
[[step]]
    action = "Inspect"
    n  = 1000 # how many reads
    infix = "inspect_at_point" # output filename infix
    target = "Read1" # Read1|Read2|Index1|Index2
    suffix = "compressed" # (optional) custom suffix for filename
    format = "gzip" # (optional) compression format: raw, gzip, zstd (defaults to raw)
    compression_level = 6 # (optional) compression level for future use
```

Output filename pattern:
- Without custom suffix: `{prefix}_{infix}_{target}.{format_extension}`
- With custom suffix: `{prefix}_{infix}_{target}_{suffix}.{format_extension}`

Where `{format_extension}` is:
- `fq` for raw format
- `fq.gz` for gzip format  
- `fq.zst` for zstd format


