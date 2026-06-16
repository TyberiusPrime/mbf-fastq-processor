---
title: Output BAM
weight: 72
---

# OutputBAM

Write reads to BAM file(s) as a pipeline step. This is the step-based equivalent
of the legacy `[output]` section with `format = "bam"`.

```toml
[input]
    read1 = "input.fq"

[[step]]
    action = "OutputBAM"
    output = ["read1"]            # segments to write to individual files (alias: segments). Defaults to all input segments.
    suffix = "bam"                # (optional) override the file suffix
    compression_level = 6         # (optional) BGZF compression level 0-9
    # interleave = ["read1","read2"]  # segments to interleave into one file (alias: interleaved)
    # chunksize = 1000000         # (optional) split into chunks of N molecules
    output_hash_compressed = false    # write a compressed-content hash sidecar
    # [step.bam] options: comment_separation_char, tags, tag_to_reference, merge_demultiplexed

[output]
    prefix = "output"
    compression_threads = 1       # output worker threads (per bam file)
```

The `bam` table accepts the same options as `[output.bam]`
(`comment_separation_char`, tag exports, `tag_to_reference`,
`merge_demultiplexed`).

> Note: tag export, reference assignment and merging of demultiplexed BAMs are
> accepted and round-tripped, but their application is part of the pipeline
> migration follow-up; only `comment_separation_char` is applied today.
