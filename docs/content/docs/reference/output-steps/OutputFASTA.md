---
title: Output FASTA
weight: 71
---

# OutputFASTA

Write reads to FASTA file(s) as a pipeline step. Like
[OutputFASTQ](../outputfastq/) but emits FASTA records (no quality line). This is
the step-based equivalent of the legacy `[output]` section with
`format = "fasta"`.

```toml
[input]
    read1 = "input.fq"

[[step]]
    action = "OutputFASTA"
    output = ["read1"]            # segments to write to individual files (alias: segments). Defaults to all input segments.
    suffix = "fasta"              # (optional) override the file suffix
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
```

Files are named like [OutputFASTQ](../outputfastq/), but with the FASTA suffix.
The output `prefix` is still taken from the `[output]` section.


## Chunking

When `chunksize` is set, start a new file every N molecules.
File names will end on .<number>, left padded with zeros to 
the actual needed number of digits.
