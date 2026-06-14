---
title: Output FASTQ
weight: 70
---

# OutputFASTQ

Write reads to FASTQ file(s) as a pipeline step. This is the step-based
equivalent of the legacy `[output]` section with `format = "fastq"`, and can be
placed and ordered like any other step. Multiple output steps are allowed.

```toml
[input]
    read1 = "input.fq"

[[step]]
    action = "OutputFASTQ"
    output = ["read1"]            # segments to write to individual files (alias: segments). Defaults to all input segments.
    suffix = "fq"                 # (optional) override the file suffix
    compression = "Raw"           # Raw / Gzip / Zstd
    # compression_level = 6       # (optional) gzip 0-9, zstd 1-22
    compression_threads = 1       # compression worker threads
    stdout = false                # write a single interleaved stream to stdout
    # interleave = ["read1","read2"]  # segments to interleave into one file (alias: interleaved)
    # chunksize = 1000000         # (optional) split into chunks of N molecules
    output_hash_uncompressed = false  # write an uncompressed-content hash sidecar
    output_hash_compressed = false    # write a compressed-content hash sidecar

[output]
    prefix = "output"
    output = []   # the OutputFASTQ step produces the read files
```

Per-segment files are named `{prefix}_{segment}.{suffix}`; with demultiplexing,
`{prefix}_{segment}_{barcode}.{suffix}`. The interleaved file is named
`{prefix}_interleaved.{suffix}`.

The output `prefix` is still taken from the `[output]` section.
